use crate::{Dependencies, Error, Result, normalize_separators};
use clap::Parser;
use std::path::{Path, PathBuf};
use tyt_injection::{parse_json, serde_json::Value};

/// Copies `settings.json` (and `settings.local.json`, if present) from one
/// profile's directory to another, plus any files referenced by string
/// values within those settings that resolve to existing files relative to
/// the source directory. Existing files in the destination are overwritten.
#[derive(Clone, Debug, Parser)]
#[command(name = "copy-profile-settings")]
pub struct CopyProfileSettings {
    /// Source profile name.
    #[arg(value_name = "from")]
    pub from: String,

    /// Destination profile name.
    #[arg(value_name = "to")]
    pub to: String,
}

const SETTINGS_FILES: &[&str] = &["settings.json", "settings.local.json"];

impl CopyProfileSettings {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let resolved = dependencies.claude_prefs()?;
        let from_dir = PathBuf::from(resolved.profiles.get(&self.from).ok_or_else(|| {
            Error::ProfileNotFound {
                name: self.from.clone(),
            }
        })?);
        let to_dir = PathBuf::from(resolved.profiles.get(&self.to).ok_or_else(|| {
            Error::ProfileNotFound {
                name: self.to.clone(),
            }
        })?);

        let canon_from = from_dir.canonicalize().ok();
        let mut copied: Vec<PathBuf> = Vec::new();

        for filename in SETTINGS_FILES {
            let src_settings = from_dir.join(filename);
            let Some(bytes) = dependencies.read_file(&src_settings)? else {
                continue;
            };
            let value: Value = parse_json(&bytes)?;

            let mut refs: Vec<String> = Vec::new();
            collect_string_values(&value, &mut refs);

            for s in refs {
                let trimmed = s.trim();
                if !trimmed.contains('/') && !trimmed.contains('\\') {
                    continue;
                }
                if Path::new(trimmed).is_absolute() {
                    continue;
                }
                let candidate = from_dir.join(trimmed);
                if !candidate.is_file() {
                    continue;
                }
                let canon_candidate = match candidate.canonicalize() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let safe = canon_from
                    .as_ref()
                    .is_some_and(|f| canon_candidate.starts_with(f));
                if !safe {
                    continue;
                }
                let dst = to_dir.join(trimmed);
                dependencies.copy_file(&candidate, &dst)?;
                copied.push(dst);
            }

            let dst_settings = to_dir.join(filename);
            dependencies.copy_file(&src_settings, &dst_settings)?;
            copied.push(dst_settings);
        }

        let mut buf = String::new();
        if copied.is_empty() {
            buf.push_str(&format!(
                "no settings to copy from {}\n",
                normalize_separators(&from_dir.to_string_lossy())
            ));
        } else {
            buf.push_str(&format!(
                "copied {} files from {} to {}:\n",
                copied.len(),
                normalize_separators(&from_dir.to_string_lossy()),
                normalize_separators(&to_dir.to_string_lossy()),
            ));
            for p in &copied {
                buf.push_str(&format!(
                    "  {}\n",
                    normalize_separators(&p.to_string_lossy())
                ));
            }
        }
        dependencies.write_stdout(buf.as_bytes())?;
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
