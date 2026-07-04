use clap::ValueEnum;

/// Clustering algorithm for `palette quantize`.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QuantizeMethod {
    /// Recursively split the color box along its longest axis.
    #[value(name = "median-cut")]
    MedianCut,

    /// Cluster through an octree over the color cube.
    #[value(name = "octree")]
    Octree,

    /// Iteratively refine k clusters by nearest centroid.
    #[value(name = "kmeans")]
    Kmeans,
}
