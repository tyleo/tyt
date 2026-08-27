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
    Dependencies as _, DependenciesImpl as PrefsDependenciesImpl, DirPrefs, JsoncCodec,
    load_sources_prefs, read_section, resolve_git_root_dir, resolve_prefs_paths,
    resolve_user_home_dir, write_section,
};
use tyt_injection::serde_json::Value;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn user_home_dir(&self) -> Result<Option<PathBuf>> {
        Ok(resolve_user_home_dir())
    }

    fn git_root_dir(&self) -> Result<Option<PathBuf>> {
        Ok(resolve_git_root_dir()?)
    }

    fn claude_prefs(&self) -> Result<ResolvedClaudePrefs> {
        let paths = resolve_prefs_paths()?;

        let sources: Vec<(&Path, &str)> = [
            (paths.user.as_deref(), ".tytconfig"),
            (paths.git_root.as_deref(), ".tytconfig"),
            (paths.git_root.as_deref(), ".tytusrconfig"),
        ]
        .into_iter()
        .filter_map(|(dir, file_name)| Some((dir?, file_name)))
        .collect();

        let layers = load_sources_prefs::<ClaudePrefs>(
            &PrefsDependenciesImpl,
            &JsoncCodec,
            &sources,
            CLAUDE_PREFS_KEY,
        )?;

        let mut resolved = ResolvedClaudePrefs::default();
        for DirPrefs { dir, prefs } in layers {
            for (k, v) in prefs.profiles {
                let resolved_path = if Path::new(&v).is_absolute() {
                    normalize_separators(&v)
                } else {
                    normalize_separators(&dir.join(&v).to_string_lossy())
                };
                resolved.profiles.insert(k, resolved_path);
            }
            if let Some(active) = prefs.active {
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
