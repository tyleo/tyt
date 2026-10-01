use crate::{RenderLight, RenderScene, RenderShadow, test_utilities::lit_room};
use ty_math::{TyLinSrgbF64, TyVector3F64};

/// The room under one point light near the corner with `shadow` and a range
/// that ends inside the room, so the falloff and the pillar's shadow both
/// show.
pub fn room_scene(shadow: RenderShadow) -> RenderScene {
    lit_room(RenderLight::Point {
        position: TyVector3F64::new(2.5, 4.0, 2.5),
        color: TyLinSrgbF64::new(1.0, 0.9, 0.7),
        strength: 40.0,
        range: Some(7.0),
        shadow,
    })
}
