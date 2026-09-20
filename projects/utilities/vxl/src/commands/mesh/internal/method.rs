use crate::CliValue;
use voxsmith::operations::mesh::Method;

impl CliValue for Method {
    const VARIANTS: &'static [Self] = &[Method::Culled, Method::Greedy, Method::Naive];

    fn name(self) -> &'static str {
        match self {
            Method::Culled => "culled",
            Method::Greedy => "greedy",
            Method::Naive => "naive",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Method::Culled => "One unmerged quad per solid-empty boundary face",
            Method::Greedy => "Coplanar faces merged into the fewest quads the run's values allow",
            Method::Naive => "All six faces of every solid voxel, hidden interior faces included",
        }
    }
}
