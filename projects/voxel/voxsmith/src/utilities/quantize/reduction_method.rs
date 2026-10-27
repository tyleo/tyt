/// Clustering algorithm quantizing groups materials with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReductionMethod {
    /// Recursively split the widest box at the median along its widest axis.
    MedianCut,

    /// Cluster through an octree over the point cube. Needs 3D points.
    Octree,

    /// Iteratively refine k clusters by nearest centroid.
    Kmeans,
}
