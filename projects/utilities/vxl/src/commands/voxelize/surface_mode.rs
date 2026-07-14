use clap::ValueEnum;

/// How `voxelize` decides which cells the surface occupies, independent of
/// `--fill-mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SurfaceMode {
    /// Fill a cell when its center lies inside the surface. Expects a closed
    /// mesh.
    #[value(name = "center-inside")]
    CenterInside,

    /// Fill a cell when any triangle passes through it. Handles an open mesh,
    /// but marks both sides of a face that lands on a cell boundary.
    #[value(name = "triangle-cover")]
    TriangleCover,
}
