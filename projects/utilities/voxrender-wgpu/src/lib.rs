#![deny(rustdoc::broken_intra_doc_links)]

//! The realtime GPU renderer for `voxrender`'s render contract, through
//! wgpu. It runs on a device its caller opens.

// Public API

mod error;
mod gpu_walk;
mod hit_trace;
mod pixel_hit;
mod result;

pub use error::*;
pub use gpu_walk::*;
pub use hit_trace::*;
pub use pixel_hit::*;
pub use result::*;
