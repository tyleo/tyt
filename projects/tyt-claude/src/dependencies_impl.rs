use crate::{CLAUDE_PREFS_KEY, ClaudePrefs, Dependencies, Error, ResolvedClaudePrefs, Result};
use std::{
    ffi::OsString,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tyt_preferences::{
    Dependencies as PrefsDependencies, DependenciesImpl as PrefsDependenciesImpl, read_section,
    write_section,
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
        let user_path = prefs_deps.user_home_dir()?.map(|d| d.join(".tytconfig"));
        let git_root = prefs_deps.git_root_dir()?;
        let git_root_path = git_root.as_ref().map(|d| d.join(".tytconfig"));
        let git_user_path = git_root.as_ref().map(|d| d.join(".tytusrconfig"));

        let mut resolved = ResolvedClaudePrefs::default();
        for source in [user_path, git_root_path, git_user_path]
            .into_iter()
            .flatten()
        {
            let Some(layer): Option<ClaudePrefs> =
                read_section(&prefs_deps, &source, CLAUDE_PREFS_KEY)?
            else {
                continue;
            };
            for (k, v) in layer.profiles {
                let resolved_path = match source.parent() {
                    Some(base) if !Path::new(&v).is_absolute() => {
                        normalize_separators(&base.join(&v).to_string_lossy())
                    }
                    _ => normalize_separators(&v),
                };
                resolved.profiles.insert(k, resolved_path);
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

/// Rewrites path separators to the platform-native form. On Windows, `/` is
/// converted to `\`. On Unix this is a no-op (backslash is a legal filename
/// character).
fn normalize_separators(s: &str) -> String {
    if cfg!(windows) {
        s.replace('/', "\\")
    } else {
        s.to_string()
    }
}
