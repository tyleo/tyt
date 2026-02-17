use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use tyt_image::{DependenciesImpl, TytImage};

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
    TytImage(TytImage),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Completion { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "tyt-image",
                &mut std::io::stdout(),
            );
        }
        Command::TytImage(image) => {
            if let Err(e) = image.execute(DependenciesImpl) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
    }
}
