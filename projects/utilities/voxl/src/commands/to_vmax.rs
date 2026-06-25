use crate::{ColorFormat, Dependencies, Format, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a voxel file to the Voxel Max format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-vmax")]
pub struct ToVmax {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// The output `.vmax` package directory to create.
    #[arg(value_name = "output")]
    output: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Where to store object colors in the package.
    #[arg(value_name = "color-format", long, default_value = "png")]
    color_format: ColorFormat,
}

impl ToVmax {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.to_vmax(&self.input, self.from, &self.output, self.color_format)
    }
}
