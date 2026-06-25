use crate::{Dependencies, Result, utilities};
use clap::Parser;

/// Finds files using .gitignore style syntax
#[derive(Clone, Debug, Parser)]
#[command(name = "find")]
pub struct Find {
    #[arg(value_name = "pattern", required = true)]
    patterns: Vec<String>,
}

impl Find {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let stdout = utilities::find_files(&dependencies, &self.patterns)?;
        if !stdout.is_empty() {
            dependencies.write_stdout(&stdout)?;
        }
        Ok(())
    }
}
