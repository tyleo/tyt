/// How a voxelized mesh fills the grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillMode {
    /// Rasterize the surface and flood-fill the volume it encloses, producing a
    /// filled body. Expects a watertight mesh.
    #[default]
    Solid,

    /// Rasterize only the voxels the triangles pass through, leaving a hollow
    /// shell.
    Surface,
}
