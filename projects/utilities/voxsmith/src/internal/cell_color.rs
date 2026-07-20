use branded_id::U32Id;
use voxcore::BVoxVoxel;

/// A per-voxel color read, chosen once per object by
/// [`resolve_cell_color`](crate::resolve_cell_color) so voxel loops carry no
/// palette machinery.
pub type CellColor<'a> = Box<dyn Fn(U32Id<BVoxVoxel>) -> [u8; 4] + 'a>;
