use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a voxel file to the mvox format.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-mvox")]
pub struct ToMvox {}

impl ToMvox {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-mvox!\n")?;
        Ok(())
    }
}
