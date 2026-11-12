use crate::{
    Error, EyeSwapchain, FrameTimes, GalleryInput, GpuClock, Result, RunOptions, XrGpu, XrSystem,
    probe_focal_lengths, probe_session, probe_visibility_mask,
};
use openxr::{
    CompositionLayerProjection, CompositionLayerProjectionView, Duration as XrDuration,
    EnvironmentBlendMode, Event, EventDataBuffer, FrameStream, FrameWaiter, Offset2Di, Posef,
    Rect2Di, ReferenceSpaceType, Session, SessionState, Space, SwapchainSubImage,
    ViewConfigurationType, ViewStateFlags, Vulkan as XrVulkan,
};
use std::{
    num::NonZeroU32,
    path::Path,
    time::{Duration, Instant},
};
use voxrender_bench::{
    Error as BenchError, HeadsetEye, HeadsetStand, HeadsetView, KernelProfile, ModelGallery,
    PassTimer, TimedBench, bench_scene,
};
use wgpu::{
    Color, CommandEncoderDescriptor, Device, Extent3d, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureDescriptor, TextureDimension, TextureUsages, TextureView, TextureViewDescriptor,
};

/// The views the app draws.
const VIEW_TYPE: ViewConfigurationType = ViewConfigurationType::PRIMARY_STEREO;

/// The display rate the app asks for, the rate the frame budget assumes.
const REFRESH_RATE: f32 = 90.0;

/// The frames between two logs of the view's time on the CPU.
const VIEW_FRAMES: u32 = 600;

/// The directory of the gallery's models within the app's files.
const MODELS: &str = "models";

/// The most seconds one frame moves a gallery's wearer by. A longer gap
/// between two frames is a stall.
const STEP: f64 = 0.1;

/// The color the pass clears both eyes to.
const CLEAR: Color = Color {
    r: 0.02,
    g: 0.04,
    b: 0.08,
    a: 1.0,
};

/// An OpenXR session that runs B0's session probes. Each frame clears both
/// eyes and times B2's kernels or runs the profile, or draws a scene's
/// view or a gallery's model.
pub struct XrApp {
    // Fields drop in order, so the OpenXR handles go before the Vulkan
    // device they use.
    swapchain: EyeSwapchain,

    frame_times: Option<FrameTimes>,

    work: FrameWork,

    space: Space,

    frame_stream: FrameStream<XrVulkan>,

    frame_waiter: FrameWaiter,

    session: Session<XrVulkan>,

    running: bool,

    eye_probes_pending: bool,

    events: EventDataBuffer,

    xr: XrSystem,

    gpu: XrGpu,
}

impl XrApp {
    /// Opens a session on the GPU and the eyes' swapchain. The frames run
    /// what `options` picks. A gallery shows the models under `files`, the
    /// app's files directory.
    pub fn new(xr: XrSystem, gpu: XrGpu, options: &RunOptions, files: &Path) -> Result<Self> {
        let blend_modes = xr
            .instance
            .enumerate_environment_blend_modes(xr.system, VIEW_TYPE)
            .map_err(Error::Xr)?;

        if !blend_modes.contains(&EnvironmentBlendMode::OPAQUE) {
            return Err(Error::NoOpaqueBlend);
        }

        // SAFETY: `session_info` holds the live handles of `gpu`, which the
        // session drops before.
        let (session, frame_waiter, frame_stream) = unsafe {
            xr.instance
                .create_session::<XrVulkan>(xr.system, &gpu.session_info)
        }
        .map_err(Error::Xr)?;

        let space = session
            .create_reference_space(ReferenceSpaceType::LOCAL, Posef::IDENTITY)
            .map_err(Error::Xr)?;

        probe_session(&xr, &session)?;

        if xr.instance.exts().fb_display_refresh_rate.is_some() {
            let rates = session
                .enumerate_display_refresh_rates()
                .map_err(Error::Xr)?;

            if rates.contains(&REFRESH_RATE) {
                session
                    .request_display_refresh_rate(REFRESH_RATE)
                    .map_err(Error::Xr)?;
            } else {
                log::info!("b0 refresh rate {REFRESH_RATE} unavailable");
            }
        }

        let frame_times = FrameTimes::new(&xr, &session)?;
        let swapchain = EyeSwapchain::new(&xr, &gpu, &session, options.scale)?;

        let width = u32::try_from(swapchain.extent.width).expect("an eye's width is positive");
        let height = u32::try_from(swapchain.extent.height).expect("an eye's height is positive");

        let work = if options.models {
            let gallery = ModelGallery::new(
                &gpu.device,
                &gpu.queue,
                &files.join(MODELS),
                options.model.as_deref(),
                swapchain.format,
                [width, height],
            )
            .map_err(Error::Bench)?;

            log::info!("models {} under {MODELS}", gallery.count());

            FrameWork::Models {
                gallery: Box::new(gallery),
                input: GalleryInput::new(&xr, &session)?,
                depth: eye_depth(&gpu.device, width, height),
                last: None,
                tour: options.tour,
                landed: None,
            }
        } else if let Some(scene) = &options.view {
            let render_scene = bench_scene(scene)
                .ok_or_else(|| Error::Bench(BenchError::NoScene(scene.clone())))?;

            let (_, view) = render_scene
                .iter_views()
                .next()
                .ok_or(Error::Bench(BenchError::NoView))?;

            FrameWork::View {
                view: Box::new(
                    HeadsetView::new(
                        &gpu.device,
                        &gpu.queue,
                        &render_scene,
                        HeadsetStand::at_view(view),
                        swapchain.format,
                        [width, height],
                    )
                    .map_err(Error::Bench)?,
                ),
                depth: eye_depth(&gpu.device, width, height),
                prepared: Duration::ZERO,
                frames: 0,
            }
        } else if options.profile {
            FrameWork::Profile(
                KernelProfile::new(
                    &gpu.device,
                    &gpu.queue,
                    width,
                    height,
                    options.kernels.as_deref(),
                )
                .map_err(Error::Bench)?,
            )
        } else {
            FrameWork::Timer {
                pass_timer: PassTimer::new(
                    &gpu.device,
                    &gpu.queue,
                    TimedBench::B2 { ray_queries: true },
                    width,
                    height,
                    options.kernels.as_deref(),
                )
                .map_err(Error::Bench)?,
                clock: GpuClock::new()?,
            }
        };

        Ok(XrApp {
            swapchain,
            frame_times,
            work,
            space,
            frame_stream,
            frame_waiter,
            session,
            running: false,
            eye_probes_pending: false,
            events: EventDataBuffer::new(),
            xr,
            gpu,
        })
    }

    /// Whether the session is between begin and end, when frames run.
    pub fn running(&self) -> bool {
        self.running
    }

    /// Handles the runtime's pending events. Returns false once the session
    /// exits.
    pub fn poll(&mut self) -> Result<bool> {
        while let Some(event) = self
            .xr
            .instance
            .poll_event(&mut self.events)
            .map_err(Error::Xr)?
        {
            match event {
                Event::SessionStateChanged(changed) => {
                    let session_state = changed.state();

                    log::info!("session {session_state:?}");

                    match session_state {
                        SessionState::READY => {
                            self.session.begin(VIEW_TYPE).map_err(Error::Xr)?;
                            self.running = true;
                        }

                        SessionState::FOCUSED => self.eye_probes_pending = true,

                        SessionState::STOPPING => {
                            self.session.end().map_err(Error::Xr)?;
                            self.running = false;
                        }

                        SessionState::EXITING | SessionState::LOSS_PENDING => return Ok(false),

                        _ => {}
                    }
                }

                Event::VisibilityMaskChangedKHR(_) => self.eye_probes_pending = true,

                Event::DisplayRefreshRateChangedFB(changed) => {
                    log::info!(
                        "b0 refresh rate: {} to {}",
                        changed.from_display_refresh_rate(),
                        changed.to_display_refresh_rate()
                    );
                }

                Event::InstanceLossPending(_) => return Ok(false),

                _ => {}
            }
        }

        Ok(true)
    }

    /// Waits for and submits the next frame, which draws only when the
    /// runtime shows it and tracks the head.
    pub fn frame(&mut self) -> Result<()> {
        let state = self.frame_waiter.wait().map_err(Error::Xr)?;

        self.frame_stream.begin().map_err(Error::Xr)?;

        let (view_flags, eyes) = self
            .session
            .locate_views(VIEW_TYPE, state.predicted_display_time, &self.space)
            .map_err(Error::Xr)?;

        // A projection layer needs real orientations, which tracking lacks
        // until it starts.
        if !state.should_render || !view_flags.contains(ViewStateFlags::ORIENTATION_VALID) {
            return self
                .frame_stream
                .end(
                    state.predicted_display_time,
                    EnvironmentBlendMode::OPAQUE,
                    &[],
                )
                .map_err(Error::Xr);
        }

        if self.eye_probes_pending {
            probe_visibility_mask(&self.xr, &self.session, &eyes)?;
            probe_focal_lengths(&self.xr, &eyes)?;
            self.eye_probes_pending = false;
        }

        let image = self.swapchain.handle.acquire_image().map_err(Error::Xr)?;
        let image = usize::try_from(image).expect("swapchain indices fit in usize");

        self.swapchain
            .handle
            .wait_image(XrDuration::INFINITE)
            .map_err(Error::Xr)?;

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("frame"),
            });

        let headset_eyes = [0, 1].map(|eye: usize| {
            let pose = eyes[eye].pose;
            let fov = eyes[eye].fov;

            HeadsetEye {
                position: [pose.position.x, pose.position.y, pose.position.z],
                rotation: [
                    pose.orientation.x,
                    pose.orientation.y,
                    pose.orientation.z,
                    pose.orientation.w,
                ],
                tangents: [
                    fov.angle_left.tan(),
                    fov.angle_right.tan(),
                    fov.angle_up.tan(),
                    fov.angle_down.tan(),
                ],
            }
        });

        // A view and a gallery draw in passes of one draw each. Any other
        // work clears the eyes in one pass.
        let passes = match &mut self.work {
            FrameWork::View {
                view,
                prepared,
                frames,
                ..
            } => {
                let start = Instant::now();

                view.prepare(&self.gpu.queue, &headset_eyes);
                *prepared += start.elapsed();
                *frames += 1;

                if *frames == VIEW_FRAMES {
                    log::info!(
                        "view prepare: {:.3} ms a frame on the CPU over {VIEW_FRAMES} frames",
                        prepared.as_secs_f64() * 1e3 / f64::from(VIEW_FRAMES)
                    );

                    *prepared = Duration::ZERO;
                    *frames = 0;
                }

                view.passes()
            }

            FrameWork::Models {
                gallery,
                input,
                last,
                tour,
                landed,
                ..
            } => {
                let mut controls = input.read(&self.session)?;
                let now = Instant::now();

                let seconds = last
                    .replace(now)
                    .map_or(0.0, |last| (now - last).as_secs_f64().min(STEP));

                // A tour steps on once its model has shown for its time.
                if let (Some(tour), Some(since)) = (*tour, *landed)
                    && (now - since).as_secs_f64() >= tour
                {
                    controls.step = 1;
                    *landed = None;
                }

                if let Some(line) = gallery.prepare(&controls, &headset_eyes, seconds) {
                    log::info!("models {line}");

                    *landed = Some(now);
                }

                gallery.passes()
            }

            _ => 1,
        };

        for at in 0..passes {
            let first = at == 0;

            let (sky, depth) = match &self.work {
                FrameWork::View { view, depth, .. } => (view.sky(), Some(depth)),
                FrameWork::Models { gallery, depth, .. } => (gallery.sky(), Some(depth)),
                _ => (CLEAR, None),
            };

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("eyes"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.swapchain.views[image],
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: if first {
                            LoadOp::Clear(sky)
                        } else {
                            LoadOp::Load
                        },
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: depth.map(|depth| RenderPassDepthStencilAttachment {
                    view: depth,
                    depth_ops: Some(Operations {
                        load: if first {
                            LoadOp::Clear(0.0)
                        } else {
                            LoadOp::Load
                        },
                        store: if at + 1 == passes {
                            StoreOp::Discard
                        } else {
                            StoreOp::Store
                        },
                    }),
                    stencil_ops: None,
                }),
                multiview_mask: NonZeroU32::new(0b11),
                ..RenderPassDescriptor::default()
            });

            match &self.work {
                FrameWork::View { view, .. } => view.draw(&mut pass, at),
                FrameWork::Models { gallery, .. } => gallery.draw(&mut pass, at),
                _ => {}
            }
        }

        match &mut self.work {
            FrameWork::View { .. } | FrameWork::Models { .. } => {
                self.gpu.queue.submit([encoder.finish()]);
            }

            FrameWork::Timer { pass_timer, clock } => {
                clock.sample()?;

                pass_timer
                    .record(&self.gpu.device, &self.gpu.queue, &mut encoder)
                    .map_err(Error::Bench)?;

                self.gpu.queue.submit([encoder.finish()]);

                if let Some(report) = pass_timer
                    .collect(&self.gpu.device, &self.gpu.queue)
                    .map_err(Error::Bench)?
                {
                    log::info!("b2 {}", clock.take());

                    for line in report.lines() {
                        log::info!("b2 {line}");
                    }
                }
            }

            FrameWork::Profile(profile) => {
                if let Some(line) = profile
                    .record(&self.gpu.device, &mut encoder)
                    .map_err(Error::Bench)?
                {
                    log::info!("profile {line}");
                }

                self.gpu.queue.submit([encoder.finish()]);
                profile.submitted(&self.gpu.queue);
            }
        }

        self.swapchain.handle.release_image().map_err(Error::Xr)?;

        let rect = Rect2Di {
            offset: Offset2Di { x: 0, y: 0 },
            extent: self.swapchain.extent,
        };

        let projection_views: Vec<_> = eyes
            .iter()
            .zip(0..)
            .map(|(eye, layer)| {
                CompositionLayerProjectionView::new()
                    .pose(eye.pose)
                    .fov(eye.fov)
                    .sub_image(
                        SwapchainSubImage::new()
                            .swapchain(&self.swapchain.handle)
                            .image_array_index(layer)
                            .image_rect(rect),
                    )
            })
            .collect();

        self.frame_stream
            .end(
                state.predicted_display_time,
                EnvironmentBlendMode::OPAQUE,
                &[&CompositionLayerProjection::new()
                    .space(&self.space)
                    .views(&projection_views)],
            )
            .map_err(Error::Xr)?;

        match &mut self.frame_times {
            Some(frame_times) => frame_times.sample(&self.session),
            None => Ok(()),
        }
    }
}

/// What each frame records after it clears the eyes.
enum FrameWork {
    /// B2's timer, which logs each GPU clock's share of a window and then the
    /// window's report on `b2` lines.
    Timer {
        pass_timer: PassTimer,

        clock: GpuClock,
    },

    /// The profile, which logs each kernel as its stretch starts on
    /// `profile` lines.
    Profile(KernelProfile),

    /// A scene's view, drawn into the eyes each frame.
    View {
        view: Box<HeadsetView>,

        /// Both eyes' depth, one layer each.
        depth: TextureView,

        /// The time the frames since the last log spent readying the view
        /// on the CPU.
        prepared: Duration,

        /// The frames since the last log.
        frames: u32,
    },

    /// A gallery's model, drawn into the eyes each frame. The app logs each
    /// model as it lands on `models` lines.
    Models {
        gallery: Box<ModelGallery>,

        input: GalleryInput,

        /// Both eyes' depth, one layer each.
        depth: TextureView,

        /// When the last frame ran.
        last: Option<Instant>,

        /// The seconds a model shows before the gallery steps on by
        /// itself.
        tour: Option<f64>,

        /// When the model in view landed, until a tour steps on from it.
        landed: Option<Instant>,
    },
}

/// A view of both eyes' depth for a `width` by `height` swapchain, one
/// layer an eye.
fn eye_depth(device: &Device, width: u32, height: u32) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: Some("eye depth"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: HeadsetView::DEPTH,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}
