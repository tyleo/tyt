use clap::ValueEnum;

/// How `voxelize` fills the grid from the mesh.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum FillMode {
    /// Rasterize the surface and flood-fill the volume it encloses, producing a
    /// filled body. Expects a watertight mesh.
    #[default]
    #[value(name = "solid")]
    Solid,
    /// Rasterize only the voxels the triangles pass through, leaving a hollow
    /// shell.
    #[value(name = "surface")]
    Surface,
}
