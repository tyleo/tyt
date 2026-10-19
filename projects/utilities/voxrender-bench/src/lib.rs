//! Benchmarks and prototypes for voxrender's GPU tiers. Each benchmark of the
//! voxrender-wgpu plan runs on a device its caller opens, judges
//! `voxrender-wgpu` and the candidates under trial against `voxrender-cpu`,
//! and returns its report.

// Public API

mod b1_timer;
mod error;
mod gpu;
mod result;
mod run_b1;

pub use b1_timer::*;
pub use error::*;
pub use gpu::*;
pub use result::*;
pub use run_b1::*;

// Internal API

mod art_room_scene;
mod cell_noise;
mod chain_kernel;
mod city_scene;
mod corner_across;
mod eye_view;
mod far_wall_scene;
mod fitted_view;
mod foliage_scene;
mod grazing_floor_scene;
mod greenhouse_scene;
mod outdoor_lights;
mod painted_object;
mod place_object;
mod reference_hits;
mod retain_matte;
mod retain_next_object;
mod walk;

pub(crate) use art_room_scene::*;
pub(crate) use cell_noise::*;
pub(crate) use chain_kernel::*;
pub(crate) use city_scene::*;
pub(crate) use corner_across::*;
pub(crate) use eye_view::*;
pub(crate) use far_wall_scene::*;
pub(crate) use fitted_view::*;
pub(crate) use foliage_scene::*;
pub(crate) use grazing_floor_scene::*;
pub(crate) use greenhouse_scene::*;
pub(crate) use outdoor_lights::*;
pub(crate) use painted_object::*;
pub(crate) use place_object::*;
pub(crate) use reference_hits::*;
pub(crate) use retain_matte::*;
pub(crate) use retain_next_object::*;
pub(crate) use walk::*;
