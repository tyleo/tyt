//! Voxel Json: the [`Voxj`] format, the writer options and their values
//! re-exported from `voxj-voxcore`, [`VoxjSerialization`] picking the
//! container, [`VoxjWriteFormat`] pairing it with the options, and the `ext`
//! block as voxconv's exts under `ext`.

// Public API

#[allow(clippy::module_inception)]
mod voxj;
mod voxj_dependencies;
mod voxj_serialization;
mod voxj_version_from_bytes;
mod voxj_write_format;

pub use voxj::*;
pub use voxj_dependencies::*;
pub use voxj_serialization::*;
pub use voxj_version_from_bytes::*;
pub use voxj_write_format::*;

pub use ::voxj::objects::{PositionEncoding, SampleEncoding};
pub use ::voxj_voxcore::{EditStateMode, VoxjWriteOptions};

// Optional API

#[cfg(feature = "ext")]
pub mod ext;

#[cfg(feature = "impl")]
mod voxj_dependencies_impl;

// Internal API

mod from_voxj_error;
