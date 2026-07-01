use crate::{Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Prints the scene graph as a tree with box-drawing glyphs, marking instanced
/// nodes and listing unplaced nodes and orphan objects.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct HierarchyShow {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Collapse repeat instances: expand a shared node's first placement and
    /// print each later placement as a non-expanded stub.
    #[arg(value_name = "collapse-instances", long = "collapse-instances")]
    collapse_instances: bool,
}

impl HierarchyShow {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.hierarchy_show(&self.input, self.from, self.collapse_instances)
    }
}
