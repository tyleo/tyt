use crate::{BFace, BSwatch, BVoxel, Groupings};
use branded_id::{IdVec, U32Id};

/// Builds `Groupings` from plain ids.
pub fn groupings(swatch_count: usize, voxel_swatches: &[u32], face_voxels: &[&[u32]]) -> Groupings {
    Groupings {
        swatch_count,
        voxel_swatches: IdVec::<BVoxel, U32Id<BSwatch>>::from(
            voxel_swatches
                .iter()
                .map(|&swatch| U32Id::from_u32(swatch))
                .collect::<Vec<_>>(),
        ),
        face_voxels: IdVec::<BFace, Vec<U32Id<BVoxel>>>::from(
            face_voxels
                .iter()
                .map(|pieces| pieces.iter().map(|&voxel| U32Id::from_u32(voxel)).collect())
                .collect::<Vec<_>>(),
        ),
    }
}
