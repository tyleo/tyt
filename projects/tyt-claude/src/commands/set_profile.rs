use crate::{Dependencies, Error, Result, Scope};
use clap::Parser;

/// Marks `<name>` as the active claude profile in the chosen `.tytconfig` /
/// `.tytusrconfig` file. The name must be defined somewhere in the cascade.
#[derive(Clone, Debug, Parser)]
#[command(name = "set-profile")]
pub struct SetProfile {
    /// Profile name to make active.
    #[arg(value_name = "name")]
    pub name: String,

    /// Which config file to write to. Defaults to `repo-user`
    /// (`<git-root>/.tytusrconfig`).
    #[arg(value_name = "scope", value_enum, conflicts_with = "scope_flag")]
    pub scope_arg: Option<Scope>,

    /// Which config file to write to. Defaults to `repo-user`
    /// (`<git-root>/.tytusrconfig`).
    #[arg(
        value_name = "scope",
        long = "scope",
        value_enum,
        conflicts_with = "scope_arg"
    )]
    pub scope_flag: Option<Scope>,
}

impl SetProfile {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let scope = self
            .scope_arg
            .or(self.scope_flag)
            .unwrap_or(Scope::RepoUser);
        let resolved = dependencies.claude_prefs()?;
        if !resolved.profiles.contains_key(&self.name) {
            return Err(Error::ProfileNotFound { name: self.name });
        }
        let target = scope.resolve_target_path(&dependencies)?;
        let mut existing = dependencies
            .read_claude_section(&target)?
            .unwrap_or_default();
        existing.active = Some(self.name.clone());
        dependencies.write_claude_section(&target, &existing)?;
        let msg = format!(
            "active claude profile in {} is now '{}'\n",
            target.display(),
            self.name
        );
        dependencies.write_stdout(msg.as_bytes())?;
        Ok(())
    }
}
