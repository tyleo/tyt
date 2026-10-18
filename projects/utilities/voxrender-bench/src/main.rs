//! Benchmarks and prototypes for voxrender's GPU tiers. Each subcommand runs
//! one benchmark of the voxrender-wgpu plan and prints its report.

use clap::{Parser, Subcommand, value_parser};
use std::process;

// Internal API

mod error;
mod gpu;
mod gpu_hits;
mod pixel_hit;
mod reference_hits;
mod result;
mod run_b1;
mod view_grid_rays;

pub(crate) use error::*;
pub(crate) use gpu::*;
pub(crate) use gpu_hits::*;
pub(crate) use pixel_hit::*;
pub(crate) use reference_hits::*;
pub(crate) use result::*;
pub(crate) use run_b1::*;
pub(crate) use view_grid_rays::*;

/// Runs voxrender's GPU benchmarks.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// B1: checks the integer walk's ids on this GPU against the reference.
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
        Command::B1 { size } => run_b1(size),
    };

    match result {
        Ok(report) => print!("{report}"),

        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}
