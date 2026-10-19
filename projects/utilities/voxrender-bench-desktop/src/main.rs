//! voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs
//! one benchmark of the voxrender-wgpu plan and prints its report.

use clap::{Parser, Subcommand, value_parser};
use std::process;
use voxrender_bench::run_b1;

// Internal API

mod error;
mod headless_gpu;
mod result;

pub(crate) use error::*;
pub(crate) use headless_gpu::*;
pub(crate) use result::*;

/// Runs voxrender's GPU benchmarks.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// B1: checks the integer walk's ids and corner shadows on this GPU
    /// against the reference.
    #[command(name = "b1")]
    B1 {
        /// The side of each view's square image.
        #[arg(
            value_name = "pixels",
            long,
            default_value_t = 64,
            value_parser = value_parser!(u32).range(1..=8192)
        )]
        size: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::B1 { size } => {
            headless_gpu().and_then(|gpu| run_b1(&gpu, size).map_err(Error::Bench))
        }
    };

    match result {
        Ok(report) => print!("{report}"),

        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}
