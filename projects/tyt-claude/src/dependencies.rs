use crate::{ClaudePrefs, ResolvedClaudePrefs, Result};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    /// Writes raw bytes to stdout.
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;

    /// Returns the user home directory, or `None` if it cannot be determined.
    fn user_home_dir(&self) -> Result<Option<PathBuf>>;

    /// Returns the root directory of the current git repository, or `None`
    /// if not in a repository.
    fn git_root_dir(&self) -> Result<Option<PathBuf>>;

    /// Resolves the cascade of `claude` sections across (most-local first):
    /// `<git-root>/.tytusrconfig`, `<git-root>/.tytconfig`, `~/.tytconfig`.
    fn claude_prefs(&self) -> Result<ResolvedClaudePrefs>;

    /// Reads just the `claude` section of a single config file at `path`.
    /// Returns `None` if the file does not exist or has no `claude` section.
    fn read_claude_section(&self, path: &Path) -> Result<Option<ClaudePrefs>>;

    /// Writes the `claude` section of a single config file at `path`,
    /// preserving any other top-level sections present in the file.
    fn write_claude_section(&self, path: &Path, prefs: &ClaudePrefs) -> Result<()>;

    /// Spawns `claude` with the supplied env vars and arguments, inheriting
    /// stdio. Returns the child's exit code.
    fn exec_claude_with_env(&self, env: &[(OsString, OsString)], args: &[OsString]) -> Result<i32>;
}
