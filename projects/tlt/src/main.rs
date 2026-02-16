use clap::Parser;
use tlt::Tlt;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Tlt,
}

fn main() {
    Cli::parse().command.execute();
}
