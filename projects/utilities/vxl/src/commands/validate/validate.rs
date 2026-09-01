use crate::{Dependencies, Result, cli_value_parser};
use clap::Parser;
use std::path::PathBuf;
use voxsmith::ValidateLayout;

/// Checks a Voxel Json document against the format spec.
#[derive(Clone, Debug, Parser)]
#[command(name = "validate")]
pub struct Validate {
    /// The Voxel Json document to validate (`.voxj` or `.voxjz`).
    #[arg(value_name = "input")]
    input: PathBuf,

    /// How to lay out the report.
    #[arg(
        value_name = "layout",
        long,
        default_value = "tables",
        value_parser = cli_value_parser::<ValidateLayout>()
    )]
    layout: ValidateLayout,
}

impl Validate {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.validate(&self.input, self.layout)
    }
}
