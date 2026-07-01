/// The strategy [`object_to_mesh_geometry`](crate::object_to_mesh_geometry) uses to turn a voxel grid
/// into quads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshMethod {
    /// Merge coplanar boundary faces into the fewest quads. Lowest triangle
    /// count.
    Greedy,

    /// One quad per solid-empty boundary face, with no merging.
    Culled,

    /// All six faces of every live voxel, including hidden interior faces.
    /// Highest triangle count.
    Naive,
}
