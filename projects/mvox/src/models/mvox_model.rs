use crate::MVoxVoxel;

/// One model: a voxel grid sized by its `SIZE` chunk and filled by the paired
/// `XYZI` chunk. A shape node references a model by its index in
/// [`MVoxFile::models`](crate::MVoxFile::models).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxModel {
    /// `[x, y, z]` grid size in voxels.
    pub size: [u32; 3],

    /// The solid voxels, in stored order.
    pub voxels: Vec<MVoxVoxel>,
}
