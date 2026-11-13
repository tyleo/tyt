//! The box raster the voxrender-wgpu plan's benchmarks picked, as a view of
//! a scene and a gallery of models. A view draws on a device its caller
//! opens. Its checks judge it against `voxrender-cpu`.

// Public API

mod bench_scene;
mod box_view;
mod box_view_shader;
mod error;
mod gallery_controls;
mod gpu;
mod headset_eye;
mod headset_stand;
mod headset_view;
mod lit_check;
mod look_from;
mod model_gallery;
mod model_scene;
mod model_tiles;
mod result;
mod retain_headset_views;
mod shape_view;
mod tile_scene;
mod view_check;
mod view_eye;
mod view_image;

pub use bench_scene::*;
pub use box_view::*;
pub use box_view_shader::*;
pub use error::*;
pub use gallery_controls::*;
pub use gpu::*;
pub use headset_eye::*;
pub use headset_stand::*;
pub use headset_view::*;
pub use lit_check::*;
pub use look_from::*;
pub use model_gallery::*;
pub use model_scene::*;
pub use model_tiles::*;
pub use result::*;
pub use retain_headset_views::*;
pub use shape_view::*;
pub use tile_scene::*;
pub use view_check::*;
pub use view_eye::*;
pub use view_image::*;

// Internal API

mod art_room_scene;
mod bench_scenes;
mod box_bindings;
mod box_bloom;
mod box_crosser;
mod box_draw;
mod box_faces;
mod box_hide;
mod box_part;
mod bush_interior_scene;
mod caster_tiles;
mod cell_grid;
mod cell_noise;
mod city_scene;
mod eye_view;
mod face_shadow;
mod fitted_view;
mod foliage_scene;
mod glass_tiles;
mod grazing_floor_scene;
mod greenhouse_scene;
mod lattice_light;
mod lit_corners;
mod lit_fill;
mod lit_page;
mod live_boxes;
mod map_readback;
mod model_rig;
mod object_quads;
mod object_tiles;
mod outdoor_lights;
mod painted_object;
mod pass_terms;
mod place_object;
mod placed_box;
mod quad_tlas;
mod retain_matte;
mod retain_next_object;
mod solid_slabs;
mod tile_masks;
mod tile_quads;
mod world_to_clip;

pub(crate) use art_room_scene::*;
pub(crate) use bench_scenes::*;
pub(crate) use box_bindings::*;
pub(crate) use box_bloom::*;
pub(crate) use box_crosser::*;
pub(crate) use box_draw::*;
pub(crate) use box_faces::*;
pub(crate) use box_hide::*;
pub(crate) use box_part::*;
pub(crate) use bush_interior_scene::*;
pub(crate) use caster_tiles::*;
pub(crate) use cell_grid::*;
pub(crate) use cell_noise::*;
pub(crate) use city_scene::*;
pub(crate) use eye_view::*;
pub(crate) use face_shadow::*;
pub(crate) use fitted_view::*;
pub(crate) use foliage_scene::*;
pub(crate) use glass_tiles::*;
pub(crate) use grazing_floor_scene::*;
pub(crate) use greenhouse_scene::*;
pub(crate) use lattice_light::*;
pub(crate) use lit_corners::*;
pub(crate) use lit_fill::*;
pub(crate) use lit_page::*;
pub(crate) use live_boxes::*;
pub(crate) use map_readback::*;
pub(crate) use model_rig::*;
pub(crate) use object_quads::*;
pub(crate) use object_tiles::*;
pub(crate) use outdoor_lights::*;
pub(crate) use painted_object::*;
pub(crate) use pass_terms::*;
pub(crate) use place_object::*;
pub(crate) use placed_box::*;
pub(crate) use quad_tlas::*;
pub(crate) use retain_matte::*;
pub(crate) use retain_next_object::*;
pub(crate) use solid_slabs::*;
pub(crate) use tile_masks::*;
pub(crate) use tile_quads::*;
pub(crate) use world_to_clip::*;

// Test support

#[cfg(test)]
mod corner_across;
#[cfg(test)]
mod reference_hits;

#[cfg(test)]
pub(crate) use corner_across::*;
#[cfg(test)]
pub(crate) use reference_hits::*;
