use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a voxel file to the goxl format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-goxl")]
pub struct ToGoxl {}

impl ToGoxl {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-goxl!\n")?;
        Ok(())
    }
}
