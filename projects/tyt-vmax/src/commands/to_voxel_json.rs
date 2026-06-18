use crate::{Dependencies, Result};
use clap::Parser;

/// Converts a .vmax file to a .voxj or .voxjz file.
#[derive(Clone, Debug, Parser)]
#[command(name = "to-voxel-json")]
pub struct ToVoxelJson {}

impl ToVoxelJson {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from to-voxel-json!\n")?;
        Ok(())
    }
}
