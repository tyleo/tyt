use crate::CliValue;
use voxsmith::ReductionMethod;

impl CliValue for ReductionMethod {
    const VARIANTS: &'static [Self] = &[
        ReductionMethod::MedianCut,
        ReductionMethod::Octree,
        ReductionMethod::Kmeans,
    ];

    fn name(self) -> &'static str {
        match self {
            ReductionMethod::MedianCut => "median-cut",
            ReductionMethod::Octree => "octree",
            ReductionMethod::Kmeans => "kmeans",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ReductionMethod::MedianCut => "Recursively split the color box along its longest axis",
            ReductionMethod::Octree => "Cluster through an octree over the color cube",
            ReductionMethod::Kmeans => "Iteratively refine k clusters by nearest centroid",
        }
    }
}
