use crate::{RenderLight, RenderScene, RenderShadow, test_utilities::lit_room};
use ty_math::{TyLinSrgbF64, TyQuaternionExt, TyQuaternionF64, TyVector3F64};

/// The room under one spot light where the point light sits, aimed at the
/// pillar with `shadow`, so the cone's disc, its soft edge, and the pillar's
/// shadow show.
pub fn spot_room_scene(shadow: RenderShadow) -> RenderScene {
    let position = TyVector3F64::new(2.5, 4.0, 2.5);
    let pillar = TyVector3F64::new(5.5, 1.5, 5.5);

    lit_room(RenderLight::Spot {
        position,
        rotation: TyQuaternionF64::from_look_direction(pillar - position, TyVector3F64::Y).unwrap(),
        color: TyLinSrgbF64::new(1.0, 0.9, 0.7),
        strength: 120.0,
        range: Some(7.0),
        inner_cone: 15f64.to_radians(),
        outer_cone: 35f64.to_radians(),
        shadow,
    })
}
