use crate::{Dependencies, Result};
use clap::Parser;
use std::path::PathBuf;

/// Converts a Voxel Json document into a `.vmax` package directory.
#[derive(Clone, Debug, Parser)]
#[command(name = "from-voxj")]
pub struct FromVoxj {
    /// The input `.voxj` or `.voxjz` document.
    #[arg(value_name = "input-voxj")]
    input_voxj: PathBuf,

    /// The output `.vmax` package directory to create.
    #[arg(value_name = "output-vmax")]
    output_vmax: PathBuf,
}

impl FromVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let voxj_bytes = dependencies.read_file(&self.input_voxj)?;
        dependencies.write_vmax_package(&voxj_bytes, &self.output_vmax)?;
        Ok(())
    }
}
