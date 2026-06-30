use crate::{Dependencies, Result};
use clap::Parser;

/// Prints one palette's selected attribute.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct Show {}

impl Show {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        dependencies.write_stdout(b"Hello from show!\n")?;
        Ok(())
    }
}
