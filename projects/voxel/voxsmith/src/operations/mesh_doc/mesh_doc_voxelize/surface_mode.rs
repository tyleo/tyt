/// How a voxelized mesh decides which cells the surface occupies, apart from
/// whether the interior is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceMode {
    /// Occupy every cell a triangle passes through. Works on any mesh, but a
    /// face on a cell boundary marks the cells on both sides.
    TriangleCover,

    /// Occupy every cell whose center lies inside the surface. Expects a closed
    /// mesh.
    CenterInside,
}
