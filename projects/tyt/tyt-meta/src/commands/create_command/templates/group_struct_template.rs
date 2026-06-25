/// Renders the source for a parent command — a `Parser` struct that delegates to
/// its `{name}Command` subcommand enum.
pub fn group_struct_template(name: &str, command: &str, description: &str) -> String {
    format!(
        r#"use crate::{{Dependencies, Result, commands::{name}Command}};
use clap::Parser;

/// {description}
#[derive(Clone, Debug, Parser)]
#[command(name = "{command}")]
pub struct {name} {{
    #[clap(subcommand)]
    pub command: {name}Command,
}}

impl {name} {{
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {{
        self.command.execute(dependencies)
    }}
}}
"#
    )
}
