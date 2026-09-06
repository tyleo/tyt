//! The Voxel Max writer options, gated behind the `vmax` feature.

mod vmax_write_options;

pub use vmax_write_options::*;

// Re-exported so a caller can name the options' values.
pub use ::vmax::VMaxSceneCamera;
pub use ::vmax_voxcore::{SceneCameraSource, VMaxColorFormat};
