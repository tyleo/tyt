use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use tyt_material::{DependenciesImpl, TytMaterial};

/// Operations on material textures.
#[derive(Clone, Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Generate shell completions.
    #[command(name = "completion")]
    Completion {
        /// The shell to generate completions for.
        #[arg(value_name = "shell")]
        shell: Shell,
    },

    #[command(flatten)]
    TytMaterial(TytMaterial),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "material", &mut io::stdout());
        }
        Command::TytMaterial(material) => {
            if let Err(e) = material.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
