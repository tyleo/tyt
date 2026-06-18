use crate::{PositionBlockSerde, SampleBlockSerde};
use serde::{Deserialize, Serialize};
use voxj::VoxjObject;

/// Serde-compatible parity type for [`VoxjObject`].
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoxjObjectSerde {
    pub name: String,
    pub palette_refs: Vec<usize>,
    pub bounds: [u32; 3],
    pub voxel_positions: PositionBlockSerde,
    pub voxel_samples: SampleBlockSerde,
}

impl From<VoxjObject> for VoxjObjectSerde {
    fn from(v: VoxjObject) -> Self {
        Self {
            name: v.name,
            palette_refs: v.palette_refs,
            bounds: v.bounds,
            voxel_positions: v.voxel_positions.into(),
            voxel_samples: v.voxel_samples.into(),
        }
    }
}

impl From<VoxjObjectSerde> for VoxjObject {
    fn from(v: VoxjObjectSerde) -> Self {
        Self {
            name: v.name,
            palette_refs: v.palette_refs,
            bounds: v.bounds,
            voxel_positions: v.voxel_positions.into(),
            voxel_samples: v.voxel_samples.into(),
        }
    }
}
