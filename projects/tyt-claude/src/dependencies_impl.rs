use crate::{CLAUDE_PREFS_KEY, ClaudePrefs, Dependencies, Error, ResolvedClaudePrefs, Result};
use std::{
    ffi::OsString,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tyt_preferences::{
    Dependencies as PrefsDependencies, DependenciesImpl as PrefsDependenciesImpl, load_git_prefs,
    load_user_git_prefs, load_user_prefs, read_section, write_section,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DependenciesImpl {
    fn prefs_deps(&self) -> PrefsDependenciesImpl {
        PrefsDependenciesImpl
    }
}

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn user_home_dir(&self) -> Result<Option<PathBuf>> {
        Ok(self.prefs_deps().user_home_dir()?)
    }

    fn git_root_dir(&self) -> Result<Option<PathBuf>> {
        Ok(self.prefs_deps().git_root_dir()?)
    }

    fn claude_prefs(&self) -> Result<ResolvedClaudePrefs> {
        let prefs_deps = self.prefs_deps();
        let user: Option<ClaudePrefs> = load_user_prefs(&prefs_deps, CLAUDE_PREFS_KEY)?;
        let git_root: Option<ClaudePrefs> = load_git_prefs(&prefs_deps, CLAUDE_PREFS_KEY)?;
        let git_user: Option<ClaudePrefs> = load_user_git_prefs(&prefs_deps, CLAUDE_PREFS_KEY)?;

        let mut resolved = ResolvedClaudePrefs::default();
        for layer in [user, git_root, git_user].into_iter().flatten() {
            for (k, v) in layer.profiles {
                resolved.profiles.insert(k, v);
            }
            if let Some(active) = layer.active {
                resolved.active = Some(active);
            }
        }
        Ok(resolved)
    }

    fn read_claude_section(&self, path: &Path) -> Result<Option<ClaudePrefs>> {
        Ok(read_section(&self.prefs_deps(), path, CLAUDE_PREFS_KEY)?)
    }

    fn write_claude_section(&self, path: &Path, prefs: &ClaudePrefs) -> Result<()> {
        Ok(write_section(
            &self.prefs_deps(),
            path,
            CLAUDE_PREFS_KEY,
            prefs,
        )?)
    }

    fn exec_claude_with_env(&self, env: &[(OsString, OsString)], args: &[OsString]) -> Result<i32> {
        match tyt_injection::exec_with_env_inherit("claude", args, env) {
            Ok(code) => Ok(code),
            Err(e) if e.kind() == ErrorKind::NotFound => Err(Error::ClaudeNotFound),
            Err(e) => Err(Error::IO(e)),
        }
    }
}
