#![deny(rustdoc::broken_intra_doc_links)]

//! The realtime GPU renderer for `voxrender`'s render contract, through
//! wgpu. It runs on a device its caller opens.

// Public API

mod dense_cells;
mod error;
mod hit_trace;
mod pixel_hit;
mod result;

pub use dense_cells::*;
pub use error::*;
pub use hit_trace::*;
pub use pixel_hit::*;
pub use result::*;
