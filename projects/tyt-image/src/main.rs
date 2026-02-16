use clap::Parser;
use tyt_image::{DependenciesImpl, TytImage};

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TytImage,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
