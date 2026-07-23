use crate::{Dependencies, Format, Result, commands::InfoLayout};
use clap::Parser;
use std::path::PathBuf;

/// Reports what a document contains, surfacing the format internals.
#[derive(Clone, Debug, Parser)]
#[command(name = "info")]
pub struct Info {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// How to lay out the report.
    #[arg(value_name = "layout", long, default_value = "tables")]
    layout: InfoLayout,
}

impl Info {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.info(&self.input, self.from, self.layout)
    }
}
