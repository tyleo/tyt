//! voxrender's GPU benchmarks as a headless desktop app. Each subcommand runs
//! one benchmark of the voxrender-wgpu plan and prints its report.

use clap::{ArgAction, Args, Parser, Subcommand, value_parser};
use std::process;
use voxrender_bench::{TimedBench, run_b1};

// Internal API

mod error;
mod headless_gpu;
mod result;
mod time_passes;

pub(crate) use error::*;
pub(crate) use headless_gpu::*;
pub(crate) use result::*;
pub(crate) use time_passes::*;

/// Runs voxrender's GPU benchmarks.
#[derive(Clone, Debug, Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    /// B1: checks the walks' and ray queries' ids and the walks' corner
    /// shadows on this GPU against the reference.
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

    /// B1: times the integer and float walks and the i32 and f32 chains on
    /// this GPU.
    #[command(name = "b1-times")]
    B1Times(Times),

    /// B2: times fixed-step loops, launch-only kernels, and the tree walk's
    /// and ray queries' ray rates on this GPU.
    #[command(name = "b2-times")]
    B2Times {
        #[command(flatten)]
        times: Times,

        /// Times the ray queries. `--ray-queries false` leaves them out for a
        /// Metal capture.
        #[arg(
            value_name = "ray-queries",
            long,
            default_value_t = true,
            default_missing_value = "true",
            num_args = 0..=1,
            action = ArgAction::Set
        )]
        ray_queries: bool,
    },
}

#[derive(Clone, Debug, Args)]
struct Times {
    /// The width of each eye's image.
    #[arg(
        value_name = "pixels",
        long,
        default_value_t = 1680,
        value_parser = value_parser!(u32).range(1..=8192)
    )]
    width: u32,

    /// The height of each eye's image.
    #[arg(
        value_name = "pixels",
        long,
        default_value_t = 1760,
        value_parser = value_parser!(u32).range(1..=8192)
    )]
    height: u32,

    /// The windows to report.
    #[arg(value_name = "count", long, default_value_t = 1)]
    windows: usize,

    /// Then captures one pass of each kernel in a graphics debugger, such as
    /// Xcode's Metal capture.
    #[arg(value_name = "capture", long)]
    capture: bool,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::B1 { size } => {
            headless_gpu().and_then(|gpu| run_b1(&gpu, size).map_err(Error::Bench))
        }

        Command::B1Times(times) => time(TimedBench::B1, &times),

        Command::B2Times { times, ray_queries } => time(TimedBench::B2 { ray_queries }, &times),
    };

    match result {
        Ok(report) => print!("{report}"),

        Err(error) => {
            eprintln!("error: {error}");
            process::exit(1);
        }
    }
}

fn time(bench: TimedBench, times: &Times) -> Result<String> {
    headless_gpu().and_then(|gpu| {
        time_passes(
            &gpu,
            bench,
            times.width,
            times.height,
            times.windows,
            times.capture,
        )
    })
}
