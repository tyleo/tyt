use crate::{Dependencies, Error, Result, Scope, normalize_separators};
use clap::Parser;

/// Adds a profile entry (name and directory) to the chosen `.tytconfig` /
/// `.tytusrconfig` file. Does not create the directory; Claude Code will
/// create it on first launch.
#[derive(Clone, Debug, Parser)]
#[command(name = "add-profile")]
pub struct AddProfile {
    /// Profile name (e.g., `work`, `personal`).
    #[arg(value_name = "name")]
    pub name: String,

    /// Directory to use as `CLAUDE_CONFIG_DIR` for this profile.
    #[arg(value_name = "path")]
    pub path: String,

    /// Which config file to write to. Defaults to `user` (`~/.tytconfig`).
    #[arg(value_name = "scope", value_enum, conflicts_with = "scope_flag")]
    pub scope_arg: Option<Scope>,

    /// Which config file to write to. Defaults to `user` (`~/.tytconfig`).
    #[arg(
        value_name = "scope",
        long = "scope",
        value_enum,
        conflicts_with = "scope_arg"
    )]
    pub scope_flag: Option<Scope>,
}

impl AddProfile {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scope = self.scope_arg.or(self.scope_flag).unwrap_or(Scope::User);
        let target = scope.resolve_target_path(&dependencies)?;
        let mut existing = dependencies
            .read_claude_section(&target)?
            .unwrap_or_default();
        if existing.profiles.contains_key(&self.name) {
            return Err(Error::ProfileAlreadyExists {
                name: self.name,
                scope,
            });
        }
        existing.profiles.insert(self.name.clone(), self.path);
        dependencies.write_claude_section(&target, &existing)?;
        let msg = format!(
            "added profile '{}' to {}\n",
            self.name,
            normalize_separators(&target.to_string_lossy())
        );
        dependencies.write_stdout(msg.as_bytes())?;
        Ok(())
    }
}
