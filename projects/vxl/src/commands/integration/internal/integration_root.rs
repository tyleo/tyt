use crate::{ResolvePrefsPaths, Result};
use clap::Args;
use std::{io::Error as IOError, path::PathBuf};

/// The root an integration works under.
#[derive(Clone, Debug, Args)]
pub struct IntegrationRoot {
    /// Works under the home directory for every project instead of the git
    /// root.
    #[arg(value_name = "user", long)]
    user: bool,
}

impl IntegrationRoot {
    /// The directory holding `.agents`.
    pub fn resolve(&self, dependencies: &impl ResolvePrefsPaths) -> Result<PathBuf> {
        let paths = dependencies.resolve_prefs_paths()?;

        if self.user {
            let Some(home) = paths.user else {
                return Err(IOError::other("no home directory is set").into());
            };

            return Ok(home);
        }

        let Some(git_root) = paths.git_root else {
            return Err(IOError::other(format!(
                "{} is not in a git repository; `--user` works under the home directory instead",
                paths.cwd.display()
            ))
            .into());
        };

        Ok(git_root)
    }
}
