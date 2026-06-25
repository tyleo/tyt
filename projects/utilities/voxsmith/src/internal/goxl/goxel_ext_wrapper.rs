use crate::GoxelExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the Goxel provenance under the `goxel` key of a
/// [`VoxState`](voxcore::VoxState) ext.
#[derive(Deserialize, Serialize)]
pub struct GoxelExtWrapper {
    pub goxel: GoxelExt,
}
