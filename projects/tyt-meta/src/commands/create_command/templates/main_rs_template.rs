use crate::commands::create_command::kebab_to_snake_case;

pub fn main_rs_template(command: &str, name: &str) -> String {
    let snake = kebab_to_snake_case(command);
    format!(
        r#"use clap::Parser;
use tyt_{snake}::{{DependenciesImpl, Tyt{name}}};

#[derive(Clone, Debug, Parser)]
pub struct Cli {{
    #[clap(subcommand)]
    pub command: Tyt{name},
}}

fn main() {{
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {{
        eprintln!("error: {{e}}");
        std::process::exit(1);
    }}
}}
"#
    )
}
