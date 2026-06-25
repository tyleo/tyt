pub fn main_rs_template(module: &str, root_enum: &str, command: &str, description: &str) -> String {
    format!(
        r#"use clap::{{CommandFactory, Parser, Subcommand}};
use clap_complete::Shell;
use std::{{io, process}};
use {module}::{{DependenciesImpl, {root_enum}}};

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
    {root_enum}({root_enum}),
}}

fn main() {{
    let cli = Cli::parse();
    match cli.command {{
        Command::Completion {{ shell }} => {{
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "{command}",
                &mut io::stdout(),
            );
        }}
        Command::{root_enum}(cmd) => {{
            if let Err(e) = cmd.execute(DependenciesImpl) {{
                eprintln!("error: {{e}}");
                process::exit(1);
            }}
        }}
    }}
}}
"#
    )
}
