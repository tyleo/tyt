use clap::Parser;
use tl_fbx::TlFbx;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: TlFbx,
}

fn main() {
    Cli::parse().command.execute();
}
