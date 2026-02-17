pub fn tyt_enum_template_empty(name: &str, description: &str) -> String {
    format!(
        r#"use clap::Subcommand;

/// {description}
#[derive(Clone, Debug, Subcommand)]
#[command(subcommand_value_name = "command")]
pub enum Tyt{name} {{}}

impl Tyt{name} {{
    pub fn execute(self, _dependencies: impl crate::Dependencies) -> crate::Result<()> {{
        match self {{}}
    }}
}}
"#
    )
}
