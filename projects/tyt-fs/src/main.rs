use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use tyt_fs::{DependenciesImpl, TytFS};

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
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "tyt-fs",
                &mut std::io::stdout(),
            );
        }
        Command::TytFS(cmd) => {
            if let Err(e) = cmd.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
