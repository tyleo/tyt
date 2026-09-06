//! The Voxel Json writer options, gated behind the `voxj` feature.

mod voxj_serialization;
mod voxj_write_options;

pub use voxj_serialization::*;
pub use voxj_write_options::*;

// Re-exported so a caller can name the options' values.
pub use ::voxj::objects::{PositionEncoding, SampleEncoding};
pub use ::voxj_voxcore::EditStateMode;
