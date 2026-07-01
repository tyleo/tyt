/// Clustering algorithm a palette reduction groups cells by color with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReductionMethod {
    /// Recursively split the color box along its widest axis at the median.
    #[default]
    MedianCut,

    /// Cluster through an octree over the color cube.
    Octree,

    /// Iteratively refine k clusters by nearest centroid.
    Kmeans,
}
