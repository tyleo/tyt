use voxcore::{VoxMain, VoxMap};

/// The state the Voxel Json converters exchange. Its ext slot carries the
/// document's `ext` block as a voxcore value tree.
pub type VoxjVoxMain = VoxMain<Option<VoxMap>>;
