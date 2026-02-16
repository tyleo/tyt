use clap::Parser;
use ty_fbx::TyFbx;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TyFbx,
}

fn main() {
    Cli::parse().command.execute();
}
