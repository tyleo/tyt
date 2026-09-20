use crate::{BFace, BSwatch, BVoxel};
use branded_id::{IdVec, U32Id};

/// The tables the reductions and climbs walk between the domains; the corner
/// rung needs none because every face owns four corners, in face order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Groupings {
    /// Each voxel entry's swatch entry.
    pub voxel_swatches: IdVec<BVoxel, U32Id<BSwatch>>,

    /// Each face entry's voxel pieces, several where a merged face spans
    /// voxels.
    pub face_voxels: IdVec<BFace, Vec<U32Id<BVoxel>>>,
}
