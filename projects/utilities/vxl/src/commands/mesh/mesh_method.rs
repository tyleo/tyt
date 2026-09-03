use crate::CliValue;
use voxsmith::MeshMethod;

impl CliValue for MeshMethod {
    const VARIANTS: &'static [Self] = &[MeshMethod::Greedy, MeshMethod::Culled, MeshMethod::Naive];

    fn name(self) -> &'static str {
        match self {
            MeshMethod::Greedy => "greedy",
            MeshMethod::Culled => "culled",
            MeshMethod::Naive => "naive",
        }
    }

    fn help(self) -> &'static str {
        match self {
            MeshMethod::Greedy => {
                "Merge coplanar, same-material faces into the fewest quads. Lowest triangle count"
            }
            MeshMethod::Culled => "One quad per solid-empty boundary face, with no merging",
            MeshMethod::Naive => {
                "All six faces of every solid voxel, including hidden interior faces. Highest \
                 triangle count"
            }
        }
    }
}
