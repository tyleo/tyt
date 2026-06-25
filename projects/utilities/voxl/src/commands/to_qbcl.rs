use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a voxel file to the qbcl format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-qbcl")]
pub struct ToQbcl {}

impl ToQbcl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-qbcl!\n")?;
        Ok(())
    }
}
