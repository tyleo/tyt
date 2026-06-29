use crate::VoxelMaxExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the Voxel Max provenance under the `voxel-max`
/// key of a [`VoxMain`](voxcore::VoxMain) ext.
#[derive(Deserialize, Serialize)]
pub struct VoxelMaxExtWrapper {
    #[serde(rename = "voxel-max")]
    pub voxel_max: VoxelMaxExt,
}
