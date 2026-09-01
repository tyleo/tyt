mod check_voxj_bytes;
mod from_voxj_bytes;
mod from_voxj_file;
mod to_voxj_bytes;
mod to_voxj_file;
mod to_voxj_vox_main;
mod to_voxjz_bytes;
mod voxj_dependencies;
mod voxj_file_builder;
mod voxj_version_from_bytes;

pub use check_voxj_bytes::*;
pub use from_voxj_bytes::*;
pub use from_voxj_file::*;
pub use to_voxj_bytes::*;
pub use to_voxj_file::*;
pub use to_voxj_vox_main::*;
pub use to_voxjz_bytes::*;
pub(crate) use voxj_dependencies::*;
pub use voxj_file_builder::*;
pub use voxj_version_from_bytes::*;

// Re-exported so callers can name the block encodings `VoxjFileBuilder`
// takes and the checks `check_voxj_bytes` reports.
pub use ::voxj::{
    objects::{PositionEncoding, SampleEncoding},
    validation::{VoxjCheck, VoxjCheckStatus},
};

// Re-exported so callers can name the block-form state `to_voxj_vox_main`
// produces and the edit-state policy `VoxjFileBuilder` takes.
pub use ::voxj_voxcore::{EditStateMode, VoxjVoxMain};
