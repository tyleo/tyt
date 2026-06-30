use crate::{Dependencies, Result};
use clap::Parser;

/// Palette operations.
#[derive(Clone, Debug, Parser)]
#[command(name = "palette")]
pub struct Palette {}

impl Palette {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from palette!\n")?;
        Ok(())
    }
}
