use clap::ValueEnum;

/// The strategy `mesh` uses to turn a voxel grid into quads.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum MeshMethod {
    /// Merge coplanar, same-material faces into the fewest quads. Lowest
    /// triangle count.
    #[value(name = "greedy")]
    Greedy,

    /// One quad per solid-empty boundary face, with no merging.
    #[value(name = "culled")]
    Culled,

    /// All six faces of every solid voxel, including hidden interior faces.
    /// Highest triangle count.
    #[value(name = "naive")]
    Naive,
}
