use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use tyt_material::{DependenciesImpl, TytMaterial};

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
    TytMaterial(TytMaterial),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "tyt-material",
                &mut std::io::stdout(),
            );
        }
        Command::TytMaterial(material) => {
            if let Err(e) = material.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
