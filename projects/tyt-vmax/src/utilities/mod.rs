// The quaternion helpers are only used by the dependency implementation's
// vmax<->voxj transform translation, so they are gated with it.
#[cfg(feature = "impl")]
mod axis_angle_to_quat;
mod color_format;
#[cfg(feature = "impl")]
mod quat_rotate;
mod voxel_max_scene_node;
mod voxj_encoding;
mod voxj_format;
mod voxj_optimize;
mod voxj_position_encoding;
mod voxj_sample_encoding;

#[cfg(feature = "impl")]
pub(crate) use axis_angle_to_quat::*;
pub use color_format::*;
#[cfg(feature = "impl")]
pub(crate) use quat_rotate::*;
pub use voxel_max_scene_node::*;
pub use voxj_encoding::*;
pub use voxj_format::*;
pub use voxj_optimize::*;
pub use voxj_position_encoding::*;
pub use voxj_sample_encoding::*;
