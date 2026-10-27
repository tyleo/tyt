//! Scenes the reference's tests and the benchmarks share: the goldens' scenes
//! and the precision scenes of the integer walk's simulation.

// Public API

mod cells_scene;
mod cube_scene;
mod glass;
mod glass_scene;
mod glow_scene;
mod l_shape_scene;
mod lit_room;
mod matte;
mod orbit_view;
mod precision_lights;
mod precision_shadow_scene;
mod precision_view_scene;
mod random_scene;
mod room_scene;
mod solid_object;
mod spot_room_scene;
mod stairs_scene;
mod two_placements_scene;

pub use cells_scene::*;
pub use cube_scene::*;
pub use glass::*;
pub use glass_scene::*;
pub use glow_scene::*;
pub use l_shape_scene::*;
pub use lit_room::*;
pub use matte::*;
pub use orbit_view::*;
pub use precision_lights::*;
pub use precision_shadow_scene::*;
pub use precision_view_scene::*;
pub use random_scene::*;
pub use room_scene::*;
pub use solid_object::*;
pub use spot_room_scene::*;
pub use stairs_scene::*;
pub use two_placements_scene::*;
