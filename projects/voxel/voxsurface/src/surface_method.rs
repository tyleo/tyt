/// The meshing strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceMethod {
    /// One unmerged quad per boundary face.
    Culled,

    /// The fewest quads the keys and the span rule allow.
    Greedy,

    /// All six faces of every solid cell.
    Naive,
}
