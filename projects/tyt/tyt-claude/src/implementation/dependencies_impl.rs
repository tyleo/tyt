use crate::{
    CLAUDE_PREFS_KEY, ClaudePrefs, Dependencies, Error, ResolvedClaudePrefs, Result,
    normalize_separators,
};
use std::{
    ffi::OsString,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use ty_preferences::{
    Dependencies as _, DependenciesImpl as PrefsDependenciesImpl, JsoncCodec, read_section,
    write_section,
};
use tyt_injection::serde_json::Value;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn user_home_dir(&self) -> Result<Option<PathBuf>> {
        Ok(PrefsDependenciesImpl.user_home_dir()?)
    }

    fn git_root_dir(&self) -> Result<Option<PathBuf>> {
        Ok(PrefsDependenciesImpl.git_root_dir()?)
    }

    fn claude_prefs(&self) -> Result<ResolvedClaudePrefs> {
        let user_path = PrefsDependenciesImpl
            .user_home_dir()?
            .map(|d| d.join(".tytconfig"));
        let git_root = PrefsDependenciesImpl.git_root_dir()?;
        let git_root_path = git_root.as_ref().map(|d| d.join(".tytconfig"));
        let git_user_path = git_root.as_ref().map(|d| d.join(".tytusrconfig"));

        let mut resolved = ResolvedClaudePrefs::default();
        for source in [user_path, git_root_path, git_user_path]
            .into_iter()
            .flatten()
        {
            let Some(layer): Option<ClaudePrefs> = read_section(
                &PrefsDependenciesImpl,
                &JsoncCodec,
                &source,
                CLAUDE_PREFS_KEY,
            )?
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
        Ok(read_section(
            &PrefsDependenciesImpl,
            &JsoncCodec,
            path,
            CLAUDE_PREFS_KEY,
        )?)
    }

    fn write_claude_section(&self, path: &Path, prefs: &ClaudePrefs) -> Result<()> {
        Ok(write_section(
            &PrefsDependenciesImpl,
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
        Ok(PrefsDependenciesImpl.read_file(path)?)
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
