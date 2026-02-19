use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use tyt_cubemap::{DependenciesImpl, TytCubemap};

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
    TytCubemap(TytCubemap),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "cubemap",
                &mut std::io::stdout(),
            );
        }
        Command::TytCubemap(cubemap) => {
            if let Err(e) = cubemap.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
