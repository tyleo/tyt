//! The voxelizer's cell mapper.

// Public API

mod map_cells;

pub use map_cells::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;
