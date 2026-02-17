use crate::{Dependencies, Result};
use clap::Parser;

/// Finds files using .gitignore style syntax
#[derive(Clone, Debug, Parser)]
#[command(name = "find")]
pub struct Find {}

impl Find {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from find!\n")?;
        Ok(())
    }
}
