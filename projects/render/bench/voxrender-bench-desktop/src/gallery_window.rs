use crate::{CenterDepth, Error, GalleryKeys, Result, WindowBlit, open_gpu};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use voxrender_bench::{GalleryControls, Gpu, ModelGallery, ViewEye, ViewTarget};
use wgpu::{
    CommandEncoderDescriptor, CurrentSurfaceTexture, Extent3d, Instance, PresentMode, Surface,
    SurfaceConfiguration, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

/// The window's title, ahead of its model and its frame rate.
const TITLE: &str = "voxrender-bench";

/// The format of the image a gallery draws.
const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// The most seconds one frame moves a gallery's wearer by. A longer gap
/// between two frames is a stall.
const STEP: f64 = 0.1;

/// A window on a [`ModelGallery`] that [`GalleryKeys`] steer. The window
/// shows the first of the two eyes the gallery draws. The image keeps the
/// size the window opens at and stretches as the window resizes.
pub struct GalleryWindow {
    directory: PathBuf,

    first: Option<String>,

    /// The width and height the window opens at, in the display's points.
    size: [u32; 2],

    detail: f64,

    seconds: Option<f64>,

    open: Option<Open>,

    /// Why the window closed, when a failure closed it.
    error: Option<Error>,
}

/// A window once it has opened.
struct Open {
    window: Arc<Window>,

    surface: Surface<'static>,

    config: SurfaceConfiguration,

    /// The surface's format with sRGB encoding.
    shown: TextureFormat,

    gpu: Gpu,

    gallery: ModelGallery,

    /// Both eyes' color and depth, one layer each.
    eyes: TextureView,

    depth: TextureView,

    center: CenterDepth,

    blit: WindowBlit,

    keys: GalleryKeys,

    /// The path of the model in view, as the gallery gives it.
    model: Option<String>,

    /// Where the pointer last was.
    pointer: Option<PhysicalPosition<f64>>,

    /// Whether the left button is down.
    dragging: bool,

    /// Whether other windows cover the window.
    hidden: bool,

    opened: Instant,

    /// When the last frame ran.
    last: Option<Instant>,

    /// The frames drawn.
    frames: u64,

    /// When the title last took its rate, the frames drawn since, and the
    /// most seconds one of them took.
    rated: (Instant, u32, f64),
}

impl GalleryWindow {
    /// Opens a window on the gallery of the models under `directory`. Once
    /// the window closes, returns a line that reports the frames drawn.
    /// Errors if the window, its device, or the gallery fails.
    ///
    /// # Arguments
    ///
    /// 1. `first`: text from the path of the model the gallery opens with
    /// 2. `size`: the width and height the window opens at, in the
    ///    display's points
    /// 3. `detail`: the gallery views' detail, or zero to draw every voxel
    /// 4. `seconds`: the seconds before the window closes by itself
    pub fn run(
        directory: &Path,
        first: Option<&str>,
        size: [u32; 2],
        detail: f64,
        seconds: Option<f64>,
    ) -> Result<String> {
        let event_loop = EventLoop::new().map_err(Error::EventLoop)?;

        let mut window = GalleryWindow {
            directory: directory.to_owned(),
            first: first.map(str::to_owned),
            size,
            detail,
            seconds,
            open: None,
            error: None,
        };

        event_loop.run_app(&mut window).map_err(Error::EventLoop)?;

        if let Some(error) = window.error {
            return Err(error);
        }

        Ok(window.open.map_or_else(String::new, |open| {
            format!(
                "{} frames in {:.1} s\n",
                open.frames,
                open.opened.elapsed().as_secs_f64()
            )
        }))
    }
}

impl ApplicationHandler for GalleryWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.open.is_some() {
            return;
        }

        match Open::new(event_loop, self) {
            Ok(open) => self.open = Some(open),

            Err(error) => {
                self.error = Some(error);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(open) = &mut self.open else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                // A window shrunk to nothing keeps its last surface.
                if size.width > 0 && size.height > 0 {
                    open.config.width = size.width;
                    open.config.height = size.height;
                    open.surface.configure(&open.gpu.device, &open.config);
                }
            }

            WindowEvent::Focused(false) => open.keys.release(),

            WindowEvent::Occluded(hidden) => open.hidden = hidden,

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if code == KeyCode::Escape {
                    event_loop.exit();
                } else {
                    open.keys.key(code, state == ElementState::Pressed, repeat);
                }
            }

            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => open.dragging = state == ElementState::Pressed,

            WindowEvent::CursorMoved { position, .. } => {
                if let Some(from) = open.pointer.replace(position)
                    && open.dragging
                {
                    let height = f64::from(open.config.height);

                    open.keys.drag(
                        (position.x - from.x) / height,
                        (position.y - from.y) / height,
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                if let Err(error) = open.frame() {
                    self.error = Some(error);
                    event_loop.exit();
                } else if self
                    .seconds
                    .is_some_and(|seconds| open.opened.elapsed().as_secs_f64() >= seconds)
                {
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(open) = &self.open
            && !open.hidden
        {
            open.window.request_redraw();
        }
    }
}

impl Open {
    fn new(event_loop: &ActiveEventLoop, window: &GalleryWindow) -> Result<Self> {
        let [width, height] = window.size;

        let attributes = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(LogicalSize::new(width, height));

        let shown_in = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(Error::Window)?,
        );

        let instance = Instance::default();

        let surface = instance
            .create_surface(shown_in.clone())
            .map_err(Error::Surface)?;

        let (adapter, gpu) = open_gpu(&instance, Some(&surface))?;

        let pixels = shown_in.inner_size();

        let mut config = surface
            .get_default_config(&adapter, pixels.width, pixels.height)
            .ok_or(Error::NoSurface)?;

        let shown = config.format.add_srgb_suffix();

        config.present_mode = PresentMode::AutoVsync;

        if shown != config.format {
            config.view_formats.push(shown);
        }

        surface.configure(&gpu.device, &config);

        let texture = |label, format, usage| {
            gpu.device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: pixels.width,
                    height: pixels.height,
                    depth_or_array_layers: 2,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };

        let color = texture(
            "gallery eyes",
            FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        );
        let depths = texture(
            "gallery depth",
            ModelGallery::DEPTH,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        );

        let first = color.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2),
            array_layer_count: Some(1),
            ..TextureViewDescriptor::default()
        });

        let gallery = ModelGallery::new(
            &gpu.device,
            &gpu.queue,
            &window.directory,
            window.first.as_deref(),
            FORMAT,
            [pixels.width, pixels.height],
            window.detail,
        )
        .map_err(Error::Bench)?;

        println!(
            "models {} under {}",
            gallery.count(),
            window.directory.display()
        );

        let now = Instant::now();

        Ok(Open {
            blit: WindowBlit::new(&gpu.device, &first, shown),
            eyes: color.create_view(&TextureViewDescriptor::default()),
            depth: depths.create_view(&TextureViewDescriptor::default()),
            center: CenterDepth::new(
                &gpu.device,
                &depths.create_view(&TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::D2),
                    aspect: TextureAspect::DepthOnly,
                    array_layer_count: Some(1),
                    ..TextureViewDescriptor::default()
                }),
            ),
            window: shown_in,
            surface,
            config,
            shown,
            gpu,
            gallery,
            keys: GalleryKeys::new(),
            model: None,
            pointer: None,
            dragging: false,
            hidden: false,
            opened: now,
            last: None,
            frames: 0,
            rated: (now, 0, 0.0),
        })
    }

    /// Steers the gallery by the keys and draws the next frame when the
    /// surface has an image to draw into.
    fn frame(&mut self) -> Result<()> {
        let now = Instant::now();

        let seconds = self
            .last
            .replace(now)
            .map_or(0.0, |last| (now - last).as_secs_f64());

        // A zoom heads for what the window's middle showed a frame or two ago.
        if let Some(depth) = self.center.read(&self.gpu.device)?
            && let Some(meters) = ViewEye::distance(depth)
            && let Some(stand) = self.gallery.stand()
        {
            self.keys.hold(meters / stand.scale);
        }

        let controls = self.keys.read();

        // The eyes take the window's shape so a stretched image stays true.
        let aspect = f64::from(self.config.width) / f64::from(self.config.height);
        let eyes = self.keys.eyes(aspect);

        if let Some(line) = self.gallery.prepare(&controls, &eyes, seconds.min(STEP)) {
            println!("models {line}");
        }

        // A model that lands draws from the head's start.
        let model = self.gallery.shown();

        if model != self.model {
            self.model = model;
            self.keys.restart();

            let eyes = self.keys.eyes(aspect);

            self.gallery
                .prepare(&GalleryControls::default(), &eyes, 0.0);
        }

        let frame = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) | CurrentSurfaceTexture::Suboptimal(frame) => {
                frame
            }

            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return Ok(()),

            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.gpu.device, &self.config);

                return Ok(());
            }

            CurrentSurfaceTexture::Lost | CurrentSurfaceTexture::Validation => {
                return Err(Error::NoFrame);
            }
        };

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("frame"),
            });

        // The depth outlasts the frame for the read of its middle.
        let target = ViewTarget {
            color: &self.eyes,
            depth: &self.depth,
            keep_depth: true,
        };

        self.gallery.encode(&mut encoder, &target);

        let surface = frame.texture.create_view(&TextureViewDescriptor {
            format: Some(self.shown),
            ..TextureViewDescriptor::default()
        });

        self.blit.draw(&mut encoder, &surface);
        self.center.copy(&mut encoder);

        self.gpu.queue.submit([encoder.finish()]);
        self.center.sent();
        self.gpu.queue.present(frame);

        self.frames += 1;

        let (since, frames, slowest) = &mut self.rated;

        *frames += 1;
        *slowest = slowest.max(seconds);

        let rated = (now - *since).as_secs_f64();

        if rated >= 1.0 {
            self.window.set_title(&format!(
                "{TITLE}: {}{:.0} frames a second, slowest {:.1} ms",
                self.model
                    .as_ref()
                    .map_or_else(String::new, |model| format!("{model}, ")),
                f64::from(*frames) / rated,
                *slowest * 1e3
            ));

            self.rated = (now, 0, 0.0);
        }

        Ok(())
    }
}
