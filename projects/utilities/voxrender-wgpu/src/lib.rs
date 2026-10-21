#![deny(rustdoc::broken_intra_doc_links)]

//! The realtime GPU renderer for `voxrender`'s render contract, through
//! wgpu. It runs on a device its caller opens.

// Public API

mod dense_cells;
mod error;
mod grid_walk_wgsl;
mod hit_trace;
mod pixel_hit;
mod placement_bytes;
mod result;

pub use dense_cells::*;
pub use error::*;
pub use grid_walk_wgsl::*;
pub use hit_trace::*;
pub use pixel_hit::*;
pub use placement_bytes::*;
pub use result::*;

// Internal API

mod push_i32s;

pub(crate) use push_i32s::*;
