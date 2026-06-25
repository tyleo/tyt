/// Renders the source for a parent command's subcommand enum — initially empty;
/// child commands are inserted by `insert_enum_variant`.
pub fn group_enum_template(name: &str, description: &str) -> String {
    format!(
        r#"use clap::Subcommand;

/// {description}
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum {name}Command {{}}

impl {name}Command {{
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {{
        match self {{}}
    }}
}}
"#
    )
}
