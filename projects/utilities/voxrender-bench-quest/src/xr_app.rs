use crate::{
    Error, EyeSwapchain, FrameTimes, GpuClock, Result, XrGpu, XrSystem, probe_focal_lengths,
    probe_session, probe_visibility_mask,
};
use openxr::{
    CompositionLayerProjection, CompositionLayerProjectionView, Duration as XrDuration,
    EnvironmentBlendMode, Event, EventDataBuffer, FrameStream, FrameWaiter, Offset2Di, Posef,
    Rect2Di, ReferenceSpaceType, Session, SessionState, Space, SwapchainSubImage,
    ViewConfigurationType, ViewStateFlags, Vulkan as XrVulkan,
};
use std::num::NonZeroU32;
use voxrender_bench::{KernelProfile, PassTimer, TimedBench};
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp,
};

/// The views the app draws.
const VIEW_TYPE: ViewConfigurationType = ViewConfigurationType::PRIMARY_STEREO;

/// The display rate the app asks for, the rate the frame budget assumes.
const REFRESH_RATE: f32 = 90.0;

/// The color the pass clears both eyes to.
const CLEAR: Color = Color {
    r: 0.02,
    g: 0.04,
    b: 0.08,
    a: 1.0,
};

/// An OpenXR session that clears both eyes each frame, runs B0's session
/// probes, and times B2's kernels or runs the profile.
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
    /// the profile when `profile`.
    pub fn new(xr: XrSystem, gpu: XrGpu, profile: bool) -> Result<Self> {
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
        let swapchain = EyeSwapchain::new(&xr, &gpu, &session)?;

        let width = u32::try_from(swapchain.extent.width).expect("an eye's width is positive");
        let height = u32::try_from(swapchain.extent.height).expect("an eye's height is positive");

        let work = if profile {
            FrameWork::Profile(
                KernelProfile::new(&gpu.device, &gpu.queue, width, height).map_err(Error::Bench)?,
            )
        } else {
            FrameWork::Timer {
                pass_timer: PassTimer::new(
                    &gpu.device,
                    &gpu.queue,
                    TimedBench::B2 { ray_queries: true },
                    width,
                    height,
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

        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("clear eyes"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.swapchain.views[image],
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(CLEAR),
                    store: StoreOp::Store,
                },
            })],
            multiview_mask: NonZeroU32::new(0b11),
            ..RenderPassDescriptor::default()
        });

        match &mut self.work {
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
                if let Some(label) = profile
                    .record(&self.gpu.device, &mut encoder)
                    .map_err(Error::Bench)?
                {
                    log::info!("profile {label}");
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
}
