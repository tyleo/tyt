use crate::CliValue;
use voxsmith::utilities::AlphaMode;

impl CliValue for AlphaMode {
    const VARIANTS: &'static [Self] =
        &[AlphaMode::Partition, AlphaMode::Distance, AlphaMode::Ignore];

    fn name(self) -> &'static str {
        match self {
            AlphaMode::Distance => "distance",
            AlphaMode::Ignore => "ignore",
            AlphaMode::Partition => "partition",
        }
    }

    fn help(self) -> &'static str {
        match self {
            AlphaMode::Distance => "Alpha adds a fourth clustering coordinate",
            AlphaMode::Ignore => "Alpha stays out; a merged voxel takes its representative's",
            AlphaMode::Partition => "Merge materials only when their alpha matches exactly",
        }
    }
}
