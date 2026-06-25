use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a voxel file to the voxj format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxj")]
pub struct ToVoxj {}

impl ToVoxj {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-voxj!\n")?;
        Ok(())
    }
}
