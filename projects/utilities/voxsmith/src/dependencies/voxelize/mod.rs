//! The voxelizer's decoder.

// Public API

mod decode_image;

pub use decode_image::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;
