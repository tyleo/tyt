mod vmax_group;
mod vmax_object;
mod vmax_scene;

// Raw `.vmaxb` / `.vmaxpsb` serde types — pure data mirroring the on-disk binary
// plists; the decode/encode algorithms live in `vmax-codec`. Gated behind `serde`
// since they exist only to (de)serialize.
#[cfg(feature = "serde")]
mod vx_brush;
#[cfg(feature = "serde")]
mod vx_brush_color;
#[cfg(feature = "serde")]
mod vx_brush_entry;
#[cfg(feature = "serde")]
mod vx_brush_state;
#[cfg(feature = "serde")]
mod vx_camera;
#[cfg(feature = "serde")]
mod vx_extent;
#[cfg(feature = "serde")]
mod vx_flag;
#[cfg(feature = "serde")]
mod vx_material;
#[cfg(feature = "serde")]
mod vx_material_palette;
#[cfg(feature = "serde")]
mod vx_mode;
#[cfg(feature = "serde")]
mod vx_object_data;
#[cfg(feature = "serde")]
mod vx_object_state;
#[cfg(feature = "serde")]
mod vx_snapshot;
#[cfg(feature = "serde")]
mod vx_snapshot_id;
#[cfg(feature = "serde")]
mod vx_stats;
#[cfg(feature = "serde")]
mod vx_storage;
#[cfg(feature = "serde")]
mod vx_tool_mode;
#[cfg(feature = "serde")]
mod vx_tools;
#[cfg(feature = "serde")]
mod vx_view_box;

pub use vmax_group::*;
pub use vmax_object::*;
pub use vmax_scene::*;

#[cfg(feature = "serde")]
pub use vx_brush::*;
#[cfg(feature = "serde")]
pub use vx_brush_color::*;
#[cfg(feature = "serde")]
pub use vx_brush_entry::*;
#[cfg(feature = "serde")]
pub use vx_brush_state::*;
#[cfg(feature = "serde")]
pub use vx_camera::*;
#[cfg(feature = "serde")]
pub use vx_extent::*;
#[cfg(feature = "serde")]
pub use vx_flag::*;
#[cfg(feature = "serde")]
pub use vx_material::*;
#[cfg(feature = "serde")]
pub use vx_material_palette::*;
#[cfg(feature = "serde")]
pub use vx_mode::*;
#[cfg(feature = "serde")]
pub use vx_object_data::*;
#[cfg(feature = "serde")]
pub use vx_object_state::*;
#[cfg(feature = "serde")]
pub use vx_snapshot::*;
#[cfg(feature = "serde")]
pub use vx_snapshot_id::*;
#[cfg(feature = "serde")]
pub use vx_stats::*;
#[cfg(feature = "serde")]
pub use vx_storage::*;
#[cfg(feature = "serde")]
pub use vx_tool_mode::*;
#[cfg(feature = "serde")]
pub use vx_tools::*;
#[cfg(feature = "serde")]
pub use vx_view_box::*;
