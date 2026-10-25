//! voxrender's GPU benchmarks as a Meta Quest OpenXR app. The crate builds
//! only for Android.

#![cfg(target_os = "android")]

// Internal API

mod android_main;
mod check_xr;
mod error;
mod eye_swapchain;
mod frame_times;
mod log_b1;
mod probe_focal_lengths;
mod probe_gpu;
mod probe_indirect;
mod probe_session;
mod probe_system;
mod probe_visibility_mask;
mod profile_mode;
mod result;
mod xr_app;
mod xr_gpu;
mod xr_system;

pub(crate) use check_xr::*;
pub(crate) use error::*;
pub(crate) use eye_swapchain::*;
pub(crate) use frame_times::*;
pub(crate) use log_b1::*;
pub(crate) use probe_focal_lengths::*;
pub(crate) use probe_gpu::*;
pub(crate) use probe_indirect::*;
pub(crate) use probe_session::*;
pub(crate) use probe_system::*;
pub(crate) use probe_visibility_mask::*;
pub(crate) use profile_mode::*;
pub(crate) use result::*;
pub(crate) use xr_app::*;
pub(crate) use xr_gpu::*;
pub(crate) use xr_system::*;
