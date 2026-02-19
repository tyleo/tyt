use crate::commands::create_command;

pub fn main_rs_template(command: &str, name: &str, description: &str) -> String {
    let snake = create_command::kebab_to_snake_case(command);
    format!(
        r#"use clap::{{CommandFactory, Parser, Subcommand}};
use clap_complete::Shell;
use tyt_{snake}::{{DependenciesImpl, Tyt{name}}};

/// {description}
#[derive(Clone, Debug, Parser)]
struct Cli {{
    #[clap(subcommand)]
    command: Command,
}}

#[derive(Clone, Debug, Subcommand)]
enum Command {{
    /// Generate shell completions.
    #[command(name = "completion")]
    Completion {{
        /// The shell to generate completions for.
        #[arg(value_name = "shell")]
        shell: Shell,
    }},

    #[command(flatten)]
    Tyt{name}(Tyt{name}),
}}

fn main() {{
    let cli = Cli::parse();
    match cli.command {{
        Command::Completion {{ shell }} => {{
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "{command}",
                &mut std::io::stdout(),
            );
        }}
        Command::Tyt{name}(cmd) => {{
            if let Err(e) = cmd.execute(DependenciesImpl) {{
                eprintln!("error: {{e}}");
                std::process::exit(1);
            }}
        }}
    }}
}}
"#
    )
}
