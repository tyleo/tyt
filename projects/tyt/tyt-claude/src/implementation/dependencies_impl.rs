use crate::{
    CLAUDE_PREFS_KEY, ClaudePrefs, Dependencies, Error, ResolvedClaudePrefs, Result,
    normalize_separators,
};
use std::{
    env,
    ffi::OsString,
    fs,
    io::{ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
};
use tyt_injection::serde_json::Value;
use tyt_preferences::{Dependencies as PrefsDependencies, JsoncCodec, read_section, write_section};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn user_home_dir(&self) -> Result<Option<PathBuf>> {
        Ok(PrefsDependencies::user_home_dir(self)?)
    }

    fn git_root_dir(&self) -> Result<Option<PathBuf>> {
        Ok(PrefsDependencies::git_root_dir(self)?)
    }

    fn claude_prefs(&self) -> Result<ResolvedClaudePrefs> {
        let user_path = PrefsDependencies::user_home_dir(self)?.map(|d| d.join(".tytconfig"));
        let git_root = PrefsDependencies::git_root_dir(self)?;
        let git_root_path = git_root.as_ref().map(|d| d.join(".tytconfig"));
        let git_user_path = git_root.as_ref().map(|d| d.join(".tytusrconfig"));

        let mut resolved = ResolvedClaudePrefs::default();
        for source in [user_path, git_root_path, git_user_path]
            .into_iter()
            .flatten()
        {
            let Some(layer): Option<ClaudePrefs> =
                read_section(self, &JsoncCodec, &source, CLAUDE_PREFS_KEY)?
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
        Ok(read_section(self, &JsoncCodec, path, CLAUDE_PREFS_KEY)?)
    }

    fn write_claude_section(&self, path: &Path, prefs: &ClaudePrefs) -> Result<()> {
        Ok(write_section(
            self,
            &JsoncCodec,
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

    fn read_file(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        Ok(PrefsDependencies::read_file(self, path)?)
    }

    fn json_string_values(&self, json: &[u8]) -> Result<Vec<String>> {
        let value: Value = tyt_injection::parse_json(json)?;

        let mut out = Vec::new();

        collect_string_values(&value, &mut out);

        Ok(out)
    }

    fn copy_file(&self, src: &Path, dst: &Path) -> Result<()> {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }
}

impl PrefsDependencies for DependenciesImpl {
    fn current_dir(&self) -> IOResult<PathBuf> {
        env::current_dir()
    }

    fn user_home_dir(&self) -> IOResult<Option<PathBuf>> {
        Ok(tyt_injection::user_home_dir())
    }

    fn git_root_dir(&self) -> IOResult<Option<PathBuf>> {
        tyt_injection::git_root_dir()
    }

    fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
        tyt_injection::read_file_optional(path)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> IOResult<()> {
        tyt_injection::write_file_atomic(path, contents)
    }
}

fn collect_string_values(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => {
            for v in a {
                collect_string_values(v, out);
            }
        }
        Value::Object(o) => {
            for v in o.values() {
                collect_string_values(v, out);
            }
        }
        _ => {}
    }
}
