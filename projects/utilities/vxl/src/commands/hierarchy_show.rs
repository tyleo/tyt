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

    /// A glob matched against node paths. When set, only matched nodes and their
    /// ancestors print. `**/` is auto-prepended unless the pattern already starts
    /// with it, so a bare pattern matches at any depth.
    #[arg(value_name = "pattern")]
    pattern: Option<String>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Collapse repeat instances: expand a shared node's first placement and
    /// print each later placement as a non-expanded stub.
    #[arg(value_name = "collapse-instances", long = "collapse-instances")]
    collapse_instances: bool,

    /// With a `pattern`, hide the ancestor chain above each match behind an
    /// `[Ancestors]` marker. No effect without a `pattern`.
    #[arg(value_name = "collapse-ancestors", long = "collapse-ancestors")]
    collapse_ancestors: bool,

    /// With a `pattern`, hide the descendants of each match behind a
    /// `[Descendants]` marker. No effect without a `pattern`.
    #[arg(value_name = "collapse-descendants", long = "collapse-descendants")]
    collapse_descendants: bool,
}

impl HierarchyShow {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.hierarchy_show(
            &self.input,
            self.from,
            self.pattern,
            self.collapse_instances,
            self.collapse_ancestors,
            self.collapse_descendants,
        )
    }
}
