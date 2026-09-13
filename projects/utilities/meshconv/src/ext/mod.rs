//! The typed path: a state carrying its format's ext as a boxed
//! [`MeshconvExt`], so a same-format write rebuilds the file exactly.
//!
//! [`FormatExt`] extends each format with its typed path. [`read_with_ext`]
//! boxes the ext the format's bridge loads into a [`MeshconvMeshMain`], and
//! [`write_with_ext`] downcasts the box back to the format's ext.
//! [`load_with_ext`] and [`save_with_ext`] start and end at a path.
//!
//! The `ext` feature, on by default, opens this module and enables every
//! enabled bridge's `serde` feature.

mod format_ext;
mod load_with_ext;
mod meshconv_ext;
mod meshconv_mesh_main;
mod read_with_ext;
mod save_with_ext;
mod write_with_ext;

pub use format_ext::*;
pub use load_with_ext::*;
pub use meshconv_ext::*;
pub use meshconv_mesh_main::*;
pub use read_with_ext::*;
pub use save_with_ext::*;
pub use write_with_ext::*;

// Shared by the formats' typed paths.

mod box_ext;

pub(crate) use box_ext::*;
