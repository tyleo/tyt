use clap::Parser;
use tyt::Tyt;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Tyt,
}

fn main() {
    Cli::parse().command.execute();
}
