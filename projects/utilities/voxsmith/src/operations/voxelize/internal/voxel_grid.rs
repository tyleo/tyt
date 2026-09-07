/// A rasterized mesh: which grid cells are filled, and the triangle each filled
/// surface cell samples, both in `x*Y*Z + y*Z + z` raster order (matching
/// [`VoxObject`](voxcore::VoxObject) voxel ids).
///
/// A surface cell (a triangle passes through it) carries `Some(triangle)`, an
/// index into the mesh's triangle table, from which its material and texture
/// coordinates are read. An interior cell a solid flood fill invents is filled
/// with no triangle (`None`), left for the caller to paint from a fill color or
/// the nearest surface.
pub(crate) struct VoxelGrid {
    /// One flag per cell: whether it is filled.
    pub filled: Vec<bool>,

    /// One entry per cell: the triangle a surface cell samples, or `None` for an
    /// empty or interior cell.
    pub triangle: Vec<Option<u32>>,
}
