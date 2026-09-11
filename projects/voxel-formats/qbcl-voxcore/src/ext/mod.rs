//! The Qubicle state with no native voxcore home, kept beside a loaded file's
//! scene so the file writes back exactly. The bare writers synthesize the file
//! from the scene instead. The typed path keeps it for each format:
//! [`QbVoxMain`], [`QbtVoxMain`], and [`QbclVoxMain`] carry their ext, the
//! `from_x_file_with_ext` loaders keep a loaded file's, and the
//! `to_x_file_with_ext` writers write it back. The exts follow the state's
//! listings through voxcore's [`VoxExt`](voxcore::VoxExt) hooks. The `serde`
//! feature derives serde for the types here.

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

mod from_qb_file_with_ext;

mod from_qbcl_file_with_ext;

mod from_qbt_file_with_ext;

mod qb_vox_main;

mod qbcl_vox_main;

mod qbt_vox_main;

mod to_qb_file_with_ext;

mod to_qbcl_file_with_ext;

mod to_qbt_file_with_ext;

pub use from_qb_file_with_ext::*;

pub use from_qbcl_file_with_ext::*;

pub use from_qbt_file_with_ext::*;

pub use qb_vox_main::*;

pub use qbcl_vox_main::*;

pub use qbt_vox_main::*;

pub use to_qb_file_with_ext::*;

pub use to_qbcl_file_with_ext::*;

pub use to_qbt_file_with_ext::*;
