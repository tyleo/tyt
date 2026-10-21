/// The WGSL types and helpers of voxrender-wgpu's walks, for a shader over
/// the placements [`placement_bytes`](crate::placement_bytes) packs. The
/// shader binds the words its walk reads as `cells`.
pub const GRID_WALK_WGSL: &str = include_str!("grid_walk.wgsl");
