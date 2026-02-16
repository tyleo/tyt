use clap::Parser;
use tyt_material::{DependenciesImpl, TytMaterial};

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TytMaterial,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
