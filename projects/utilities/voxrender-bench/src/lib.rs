//! Benchmarks and prototypes for voxrender's GPU tiers. Each benchmark of the
//! voxrender-wgpu plan runs on a device its caller opens, judges
//! `voxrender-wgpu` and the candidates under trial against `voxrender-cpu`,
//! and returns its report.

// Public API

mod error;
mod gpu;
mod result;
mod run_b1;

pub use error::*;
pub use gpu::*;
pub use result::*;
pub use run_b1::*;

// Internal API

mod corner_across;
mod reference_hits;

pub(crate) use corner_across::*;
pub(crate) use reference_hits::*;
