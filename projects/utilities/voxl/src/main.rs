use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use voxl::{DependenciesImpl, Voxl};

/// A command-line tool for working with voxels.
#[derive(Clone, Debug, Parser)]
#[command(name = "voxl")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    voxl: Voxl,
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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Completion { shell }) => {
            clap_complete::generate(shell, &mut Cli::command(), "voxl", &mut io::stdout());
        }
        None => {
            if let Err(e) = cli.voxl.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
