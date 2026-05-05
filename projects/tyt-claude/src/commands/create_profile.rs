use crate::{Dependencies, Error, Result, Scope};
use clap::Parser;

/// Adds a profile entry (name and directory) to the chosen `.tytconfig` /
/// `.tytusrconfig` file. Does not create the directory; Claude Code will
/// create it on first launch.
#[derive(Clone, Debug, Parser)]
#[command(name = "create-profile")]
pub struct CreateProfile {
    /// Profile name (e.g., `work`, `personal`).
    #[arg(value_name = "name")]
    pub name: String,

    /// Directory to use as `CLAUDE_CONFIG_DIR` for this profile.
    #[arg(value_name = "path")]
    pub path: String,

    /// Which config file to write to.
    #[arg(value_name = "scope", long, value_enum, default_value_t = Scope::default())]
    pub scope: Scope,
}

impl CreateProfile {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let target = self.scope.resolve_target_path(&dependencies)?;
        let mut existing = dependencies
            .read_claude_section(&target)?
            .unwrap_or_default();
        if existing.profiles.contains_key(&self.name) {
            return Err(Error::ProfileAlreadyExists {
                name: self.name,
                scope: self.scope,
            });
        }
        existing.profiles.insert(self.name.clone(), self.path);
        dependencies.write_claude_section(&target, &existing)?;
        let msg = format!("added profile '{}' to {}\n", self.name, target.display());
        dependencies.write_stdout(msg.as_bytes())?;
        Ok(())
    }
}
