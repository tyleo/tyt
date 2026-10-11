//! Decodes and encodes the per-chunk voxel `snapshots` of a
//! [`VMaxContentsVmaxbFile`](crate::VMaxContentsVmaxbFile), the baked edit
//! log holding an object's geometry. Decoding replays the log into
//! model-space voxels and encoding rebuilds it from them. A contents file
//! older than [`SNAPSHOT_CONTENTS_VERSION`] holds legacy chunks instead.

// Public API

mod decode_vmax_legacy_chunks;
mod decode_vmax_snapshots;
mod encode_contents_vmaxb_file_from_voxels;
mod encode_vmax_snapshots;
mod error;
mod result;
mod snapshot_contents_version;
mod vmax_voxel;

pub use decode_vmax_legacy_chunks::*;
pub use decode_vmax_snapshots::*;
pub use encode_contents_vmaxb_file_from_voxels::*;
pub use encode_vmax_snapshots::*;
pub use error::*;
pub use result::*;
pub use snapshot_contents_version::*;
pub use vmax_voxel::*;

// Internal API

mod internal;
pub(crate) use internal::*;
