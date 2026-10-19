#![deny(rustdoc::broken_intra_doc_links)]

//! The realtime GPU renderer for `voxrender`'s render contract, through
//! wgpu. It runs on a device its caller opens.

// Public API

mod error;
mod gpu_walk;
mod pixel_hit;
mod result;
mod trace_hits;

pub use error::*;
pub use gpu_walk::*;
pub use pixel_hit::*;
pub use result::*;
pub use trace_hits::*;
