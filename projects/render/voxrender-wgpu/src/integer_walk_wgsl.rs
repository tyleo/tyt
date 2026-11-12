/// The WGSL of voxrender's integer walk: `fn walk(placement: Placement,
/// ray: Ray) -> Hit` over [`GRID_WALK_WGSL`](crate::GRID_WALK_WGSL), for a
/// shader that reads each object's [`dense_cells`](crate::dense_cells) as
/// `cells`.
pub const INTEGER_WALK_WGSL: &str = include_str!("integer_walk.wgsl");
