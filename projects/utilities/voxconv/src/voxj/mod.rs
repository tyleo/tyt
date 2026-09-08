//! The Voxel Json writer options and their values, re-exported from
//! `voxj-voxcore` behind the `voxj` feature. `VoxjSerialization`, defined
//! here, picks the container.

mod voxj_serialization;

pub use voxj_serialization::*;

pub use ::voxj::objects::{PositionEncoding, SampleEncoding};
pub use ::voxj_voxcore::{EditStateMode, VoxjWriteOptions};
