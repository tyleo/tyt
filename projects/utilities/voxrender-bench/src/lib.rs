//! Benchmarks and prototypes for voxrender's GPU tiers. Each benchmark of the
//! voxrender-wgpu plan runs on a device its caller opens, judges
//! `voxrender-wgpu` and the candidates under trial against `voxrender-cpu`,
//! and returns its report.

// Public API

mod b1_timer;
mod error;
mod gpu;
mod result;
mod run_b1;

pub use b1_timer::*;
pub use error::*;
pub use gpu::*;
pub use result::*;
pub use run_b1::*;

// Internal API

mod chain_kernel;
mod corner_across;
mod far_wall_scene;
mod reference_hits;

pub(crate) use chain_kernel::*;
pub(crate) use corner_across::*;
pub(crate) use far_wall_scene::*;
pub(crate) use reference_hits::*;
