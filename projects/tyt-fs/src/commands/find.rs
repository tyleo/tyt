use crate::{Dependencies, Error, Result};
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
        let mut args = vec!["--files".to_owned()];
        for pattern in self.patterns {
            args.push("-g".to_owned());
            args.push(pattern);
        }

        match dependencies.exec_rg(args) {
            Ok(stdout) => dependencies.write_stdout(&stdout),
            Err(Error::Rg(ref e)) if e.exit_code == Some(1) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
