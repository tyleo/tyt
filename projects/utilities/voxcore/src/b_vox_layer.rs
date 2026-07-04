/// Brand marker for a layer in a [`VoxObject`](crate::VoxObject): one of the
/// object's layers, each referencing a shared [`VoxPalette`](crate::VoxPalette).
/// Two layers may reference the same palette, and layers do not merge. Each
/// voxel carries one material sample per layer.
pub struct BVoxLayer;
