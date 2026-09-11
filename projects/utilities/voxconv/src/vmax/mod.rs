//! Voxel Max: the [`VMax`] format, and the writer options and their values
//! re-exported from `vmax-voxcore` so a caller can name them.

mod from_vmax_error;
mod package_file;
#[allow(clippy::module_inception)]
mod vmax;
mod vmax_dependencies;

pub(crate) use package_file::*;
pub use vmax::*;
pub use vmax_dependencies::*;

#[cfg(feature = "impl")]
mod vmax_dependencies_impl;

#[cfg(feature = "ext")]
mod ext;

pub use ::vmax::VMaxSceneCamera;
pub use ::vmax_voxcore::{SceneCameraSource, VMaxColorFormat, VMaxWriteOptions};
