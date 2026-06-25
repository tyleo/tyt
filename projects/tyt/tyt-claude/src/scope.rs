use crate::{Dependencies, Error, Result};
use clap::ValueEnum;
use std::path::PathBuf;

/// Which config file `add-profile` / `set-profile` should write to.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Scope {
    /// `~/.tytconfig`
    User,
    /// `<git-root>/.tytconfig` (checked into the repo)
    Repo,
    /// `<git-root>/.tytusrconfig` (user-local, not checked in)
    #[default]
    RepoUser,
}

impl Scope {
    /// Resolves the absolute path of the config file this scope writes to.
    pub fn resolve_target_path(&self, dependencies: &impl Dependencies) -> Result<PathBuf> {
        match self {
            Scope::User => {
                let home = dependencies.user_home_dir()?.ok_or(Error::NoUserHome)?;
                Ok(home.join(".tytconfig"))
            }
            Scope::Repo => {
                let root = dependencies.git_root_dir()?.ok_or(Error::NoGitRoot)?;
                Ok(root.join(".tytconfig"))
            }
            Scope::RepoUser => {
                let root = dependencies.git_root_dir()?.ok_or(Error::NoGitRoot)?;
                Ok(root.join(".tytusrconfig"))
            }
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::User => f.write_str("user"),
            Scope::Repo => f.write_str("repo"),
            Scope::RepoUser => f.write_str("repo-user"),
        }
    }
}
