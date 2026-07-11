use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use std::{io, process};
use vxl::{DependenciesImpl, Error, Vxl};

/// A command-line tool for working with voxels.
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
    Vxl(Box<Vxl>),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "vxl", &mut io::stdout());
        }
        Command::Vxl(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                match e {
                    Error::Usage(clap_error) => clap_error.exit(),
                    e => {
                        eprintln!("error: {e}");
                        process::exit(1);
                    }
                }
            }
        }
    }
}
