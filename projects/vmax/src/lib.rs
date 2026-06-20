mod vmax_group;
mod vmax_object;
mod vmax_scene;
mod vmax_scene_camera;

// Raw `.vmaxb` / `.vmaxpsb` data types — pure structs mirroring the on-disk
// binary plists; the decode/encode algorithms live in `vmax-codec`. Their serde
// impls are additive (the optional `serde` feature), so the types always exist —
// with serde off they simply carry no (de)serialization.
mod vx_brush;
mod vx_brush_color;
mod vx_brush_entry;
mod vx_brush_state;
mod vx_camera;
mod vx_extent;
mod vx_extent_range;
mod vx_flag;
mod vx_flag_value;
mod vx_material;
mod vx_material_md;
mod vx_material_palette;
mod vx_mode;
mod vx_object_data;
mod vx_object_state;
mod vx_snapshot;
mod vx_snapshot_id;
mod vx_stats;
mod vx_storage;
mod vx_tool_mode;
mod vx_tools;
mod vx_view_box;

pub use vmax_group::*;
pub use vmax_object::*;
pub use vmax_scene::*;
pub use vmax_scene_camera::*;

pub use vx_brush::*;
pub use vx_brush_color::*;
pub use vx_brush_entry::*;
pub use vx_brush_state::*;
pub use vx_camera::*;
pub use vx_extent::*;
pub use vx_extent_range::*;
pub use vx_flag::*;
pub use vx_flag_value::*;
pub use vx_material::*;
pub use vx_material_md::*;
pub use vx_material_palette::*;
pub use vx_mode::*;
pub use vx_object_data::*;
pub use vx_object_state::*;
pub use vx_snapshot::*;
pub use vx_snapshot_id::*;
pub use vx_stats::*;
pub use vx_storage::*;
pub use vx_tool_mode::*;
pub use vx_tools::*;
pub use vx_view_box::*;
