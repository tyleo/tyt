use clap::Parser;
use tyt_cubemap::{DependenciesImpl, TytCubemap};

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TytCubemap,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
