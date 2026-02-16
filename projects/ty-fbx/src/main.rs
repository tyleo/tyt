use clap::Parser;
use ty_fbx::{DependenciesImpl, TyFbx};

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TyFbx,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute(DependenciesImpl) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
