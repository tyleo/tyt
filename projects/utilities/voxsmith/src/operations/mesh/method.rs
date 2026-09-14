/// The meshing strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// One unmerged quad per boundary face.
    Culled,

    /// The fewest quads the run's values allow.
    Greedy,

    /// All six faces of every solid voxel.
    Naive,
}
