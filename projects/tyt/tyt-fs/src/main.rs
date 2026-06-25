use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use tyt_fs::{DependenciesImpl, TytFS};

/// Operations on the filesystem.
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
    TytFS(TytFS),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "fs", &mut io::stdout());
        }
        Command::TytFS(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}
