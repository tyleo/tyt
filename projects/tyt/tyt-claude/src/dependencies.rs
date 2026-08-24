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

    /// Reads bytes from `path`. Returns `None` if the file does not exist.
    fn read_file(&self, path: &Path) -> Result<Option<Vec<u8>>>;

    /// Returns every string value in a JSON document, in depth-first
    /// document order.
    fn json_string_values(&self, json: &[u8]) -> Result<Vec<String>>;

    /// Copies the file at `src` to `dst`, creating parent directories of
    /// `dst` as needed. Overwrites if `dst` already exists.
    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()>;
}
