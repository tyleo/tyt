use crate::{Dependencies, Format, ReportLayout, Result};
use clap::Parser;
use std::path::PathBuf;

/// Lists every palette in a document, one row apiece.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct PaletteList {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// How to lay out the listing.
    #[arg(value_name = "layout", long, default_value = "markdown")]
    layout: ReportLayout,
}

impl PaletteList {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.palette_list(&self.input, self.from, self.layout)
    }
}
