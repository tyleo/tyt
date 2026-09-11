//! The typed path: a state carrying its format's ext as a boxed [`VoxconvExt`],
//! so a same-format write rebuilds the file exactly and a Voxel Json write
//! keeps the ext in its `ext` block.
//!
//! [`read_with_ext`] boxes the ext the format's bridge loads into a
//! [`VoxconvVoxMain`]. [`write_with_ext`] downcasts the box to the format's
//! ext. Failing that, it takes the format's entry from the `ext` block the box
//! encodes to. A box with no entry for the format writes the synthesized file
//! the bare pair writes. [`load_with_ext`] and [`save_with_ext`] start and end
//! at a path. Behind the `voxj` feature, a Voxel Json document's block decodes
//! into a `CompositeVoxExt`, with each enabled format's entry as that format's
//! ext and the rest as an `InertVoxExt`. The keys live here because voxconv is
//! where the formats meet Voxel Json. A bridge knows nothing of its key.
//!
//! The `ext` feature, on by default, opens this module and enables every
//! enabled bridge's `serde` feature for the Voxel Json transcode.

mod load_with_ext;
mod read_with_ext;
mod save_with_ext;
mod voxconv_ext;
mod voxconv_vox_main;
mod write_with_ext;

pub use load_with_ext::*;
pub use read_with_ext::*;
pub use save_with_ext::*;
pub use voxconv_ext::*;
pub use voxconv_vox_main::*;
pub use write_with_ext::*;

#[cfg(feature = "voxj")]
mod voxj;

#[cfg(feature = "voxj")]
pub use voxj::*;
