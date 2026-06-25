use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a voxel file to the vmax format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-vmax")]
pub struct ToVmax {}

impl ToVmax {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-vmax!\n")?;
        Ok(())
    }
}
