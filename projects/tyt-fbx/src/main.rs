use clap::Parser;
use tyt_fbx::{DependenciesImpl, TytFbx};

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TytFbx,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
