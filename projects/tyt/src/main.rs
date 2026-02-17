use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use tyt::{DependenciesImpl, Tyt};

#[derive(Clone, Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

/// Top-level command dispatcher.
#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// Generate shell completions.
    #[command(name = "completion")]
    Completion {
        /// The shell to generate completions for.
        shell: Shell,
    },

    #[command(flatten)]
    Tyt(Tyt),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "tyt", &mut std::io::stdout());
        }
        Command::Tyt(tyt) => {
            if let Err(e) = tyt.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
