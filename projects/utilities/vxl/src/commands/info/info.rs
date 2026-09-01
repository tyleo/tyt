use crate::{Dependencies, Format, Result, cli_value_parser};
use clap::Parser;
use std::path::PathBuf;
use voxsmith::InfoLayout;

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
    #[arg(
        value_name = "layout",
        long,
        default_value = "tables",
        value_parser = cli_value_parser::<InfoLayout>()
    )]
    layout: InfoLayout,
}

impl Info {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.info(&self.input, self.from, self.layout)
    }
}
