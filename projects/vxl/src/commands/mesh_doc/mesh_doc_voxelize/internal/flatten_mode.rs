use crate::CliValue;
use voxsmith::utilities::FlattenMode;

impl CliValue for FlattenMode {
    const VARIANTS: &'static [Self] =
        &[FlattenMode::None, FlattenMode::Nodes, FlattenMode::Objects];

    fn name(self) -> &'static str {
        match self {
            FlattenMode::None => "none",
            FlattenMode::Nodes => "nodes",
            FlattenMode::Objects => "objects",
        }
    }

    fn help(self) -> &'static str {
        match self {
            FlattenMode::None => "Keep a node for each placement, nested as the hierarchy nests",

            FlattenMode::Nodes => {
                "Write one node for each root, holding every object below the root on one grid"
            }

            FlattenMode::Objects => {
                "Write one node for each root, holding one object of every voxel below the root"
            }
        }
    }
}
