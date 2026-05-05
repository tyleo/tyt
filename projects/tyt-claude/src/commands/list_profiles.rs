use crate::{Dependencies, Result};
use clap::Parser;

/// Lists all Claude profiles found by walking the `.tytusrconfig` /
/// `.tytconfig` cascade up from the current directory through `~/.tytconfig`.
/// The active profile is marked with `*`.
#[derive(Clone, Debug, Parser)]
#[command(name = "list-profiles")]
pub struct ListProfiles {}

impl ListProfiles {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let resolved = dependencies.claude_prefs()?;
        let mut output = String::new();
        if resolved.profiles.is_empty() {
            output.push_str("no claude profiles\n");
        } else {
            let width = resolved.profiles.keys().map(String::len).max().unwrap_or(0);
            for (name, path) in &resolved.profiles {
                let marker = if resolved.active.as_deref() == Some(name.as_str()) {
                    '*'
                } else {
                    ' '
                };
                output.push_str(&format!("{marker} {name:<width$}  {path}\n"));
            }
        }
        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}
