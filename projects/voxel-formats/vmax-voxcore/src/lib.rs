#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Max packages and the voxcore state.
//!
//! The state is a [`VMaxVoxMain`], a [`VoxMain`](voxcore::VoxMain) carrying a
//! [`VMaxExt`] with the Voxel Max state that has no native voxcore home.
//! [`from_vmax_file`] loads a [`VMaxFile`](vmax::VMaxFile) into one and
//! [`to_vmax_file`] writes one back, exactly for a loaded document, with
//! [`VMaxWriteOptions`] picking the color format and the scene camera.
//! [`to_vmax_vox_main`] gives a bare `VoxMain<()>` a synthesized ext, and
//! `take_ext` takes the ext back off. The ext follows the state's listings
//! through voxcore's [`VoxExt`](voxcore::VoxExt) hooks. The `codec` module,
//! behind the default `codec` feature, goes straight to and from a package's
//! files. It takes the codec's dependencies, which
//! `vmax_codec::DependenciesImpl` supplies. The `serde` feature, on by
//! default, derives serde for the ext types.

// Public API

mod error;
mod ext;
mod from_vmax_file;
mod result;
mod scene_camera_source;
mod to_vmax_file;
mod to_vmax_vox_main;
mod vmax_color_format;
mod vmax_vox_main;
mod vmax_write_options;

pub use error::*;
pub use ext::*;
pub use from_vmax_file::*;
pub use result::*;
pub use scene_camera_source::*;
pub use to_vmax_file::*;
pub use to_vmax_vox_main::*;
pub use vmax_color_format::*;
pub use vmax_vox_main::*;
pub use vmax_write_options::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
