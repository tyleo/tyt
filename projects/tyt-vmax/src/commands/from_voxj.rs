use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a Voxel Json document into a .vmax package, written to stdout.
#[derive(Clone, Debug, Parser)]
#[command(name = "from-voxj")]
pub struct FromVoxj {}

impl FromVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from from-voxj!\n")?;
        Ok(())
    }
}
