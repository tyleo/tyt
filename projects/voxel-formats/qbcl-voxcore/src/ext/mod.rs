//! The Qubicle state with no native voxcore home, kept beside a loaded file's
//! scene so the file writes back exactly. The bare writers synthesize the
//! file from the scene instead. The `ext` feature makes this module public
//! and adds the typed path for
//! each format: [`QbVoxMain`], [`QbtVoxMain`], and [`QbclVoxMain`] carry
//! their ext, the `from_x_file_with_ext` loaders keep a loaded file's, and
//! the `to_x_file_with_ext` writers write it back. The exts follow the
//! state's listings through voxcore's [`VoxExt`](voxcore::ext::VoxExt)
//! hooks. Each ext enters a document's
//! `ext` block as its `qb`, `qbt`, or `qbcl` entry through voxcore's
//! [`VoxExtEntryCodec`](voxcore::ext::VoxExtEntryCodec).

// Types

mod qb_ext;
mod qb_ext_matrix;
mod qbcl_ext;
mod qbcl_ext_metadata;
mod qbcl_ext_node;
mod qbcl_ext_node_body;
mod qbcl_ext_thumbnail;
mod qbt_ext;
mod qbt_ext_node;

pub use qb_ext::*;
pub use qb_ext_matrix::*;
pub use qbcl_ext::*;
pub use qbcl_ext_metadata::*;
pub use qbcl_ext_node::*;
pub use qbcl_ext_node_body::*;
pub use qbcl_ext_thumbnail::*;
pub use qbt_ext::*;
pub use qbt_ext_node::*;

// Typed path

#[cfg(feature = "ext")]
mod from_qb_file_with_ext;

#[cfg(feature = "ext")]
mod from_qbcl_file_with_ext;

#[cfg(feature = "ext")]
mod from_qbt_file_with_ext;

#[cfg(feature = "ext")]
mod qb_vox_main;

#[cfg(feature = "ext")]
mod qbcl_vox_main;

#[cfg(feature = "ext")]
mod qbt_vox_main;

#[cfg(feature = "ext")]
mod to_qb_file_with_ext;

#[cfg(feature = "ext")]
mod to_qbcl_file_with_ext;

#[cfg(feature = "ext")]
mod to_qbt_file_with_ext;

#[cfg(feature = "ext")]
mod vox_ext;

#[cfg(feature = "ext")]
mod vox_ext_entry_codec;

#[cfg(feature = "ext")]
pub use from_qb_file_with_ext::*;

#[cfg(feature = "ext")]
pub use from_qbcl_file_with_ext::*;

#[cfg(feature = "ext")]
pub use from_qbt_file_with_ext::*;

#[cfg(feature = "ext")]
pub use qb_vox_main::*;

#[cfg(feature = "ext")]
pub use qbcl_vox_main::*;

#[cfg(feature = "ext")]
pub use qbt_vox_main::*;

#[cfg(feature = "ext")]
pub use to_qb_file_with_ext::*;

#[cfg(feature = "ext")]
pub use to_qbcl_file_with_ext::*;

#[cfg(feature = "ext")]
pub use to_qbt_file_with_ext::*;
