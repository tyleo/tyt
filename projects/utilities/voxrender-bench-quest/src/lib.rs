//! voxrender's GPU benchmarks as a Meta Quest OpenXR app. The crate builds
//! only for Android.

#![cfg(target_os = "android")]

// Internal API

mod android_main;
mod check_xr;
mod error;
mod eye_swapchain;
mod frame_times;
mod probe_gpu;
mod probe_indirect;
mod probe_session;
mod probe_system;
mod probe_visibility_mask;
mod result;
mod xr_app;
mod xr_gpu;
mod xr_system;

pub(crate) use check_xr::*;
pub(crate) use error::*;
pub(crate) use eye_swapchain::*;
pub(crate) use frame_times::*;
pub(crate) use probe_gpu::*;
pub(crate) use probe_indirect::*;
pub(crate) use probe_session::*;
pub(crate) use probe_system::*;
pub(crate) use probe_visibility_mask::*;
pub(crate) use result::*;
pub(crate) use xr_app::*;
pub(crate) use xr_gpu::*;
pub(crate) use xr_system::*;
