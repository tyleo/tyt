//! voxrender's GPU benchmarks as a Meta Quest OpenXR app. The crate builds
//! only for Android.

#![cfg(target_os = "android")]

// Internal API

mod android_main;
mod error;
mod eye_swapchain;
mod result;
mod xr_app;
mod xr_gpu;
mod xr_system;

pub(crate) use error::*;
pub(crate) use eye_swapchain::*;
pub(crate) use result::*;
pub(crate) use xr_app::*;
pub(crate) use xr_gpu::*;
pub(crate) use xr_system::*;
