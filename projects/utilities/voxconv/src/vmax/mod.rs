//! Voxel Max: the [`VMax`] format, and the writer options and their values
//! re-exported from `vmax-voxcore` so a caller can name them.

// Public API

#[allow(clippy::module_inception)]
mod vmax;
mod vmax_dependencies;

pub use vmax::*;
pub use vmax_dependencies::*;

pub use ::vmax::VMaxSceneCamera;
pub use ::vmax_voxcore::{SceneCameraSource, VMaxColorFormat, VMaxObjectSize, VMaxWriteOptions};

// Optional API

#[cfg(feature = "ext")]
mod vmax_format_ext;

#[cfg(feature = "impl")]
mod vmax_dependencies_impl;

// Internal API

mod from_vmax_error;
mod package_file;

pub(crate) use package_file::*;
