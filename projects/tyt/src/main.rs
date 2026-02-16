use clap::Parser;
use tyt::Tyt;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Tyt,
}

fn main() {
    if let Err(e) = Cli::parse().command.execute() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
