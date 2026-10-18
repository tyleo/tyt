use ty_math::TyVector3F64;
use voxrender::{RenderLight, RenderScene};
use voxrender_cpu::ShadowTarget;

/// Where each light of `scene` sends its shadow rays, in light order. The
/// hemisphere casts none.
pub fn shadow_targets(scene: &RenderScene) -> Vec<ShadowTarget> {
    scene
        .iter_lights()
        .filter_map(|(_, light)| match *light {
            RenderLight::Directional { rotation, .. } => {
                Some(ShadowTarget::Direction(rotation * TyVector3F64::Z))
            }

            RenderLight::Point { position, .. } | RenderLight::Spot { position, .. } => {
                Some(ShadowTarget::Position(position))
            }

            RenderLight::Hemisphere { .. } => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::shadow_targets;
    use voxrender::{RenderShadow, fixtures::spot_room_scene};
    use voxrender_cpu::ShadowTarget;

    #[test]
    fn the_hemisphere_casts_no_shadow() {
        let scene = spot_room_scene(RenderShadow::PerPixel);
        let targets = shadow_targets(&scene);

        assert_eq!(targets.len(), scene.iter_lights().count() - 1);
        assert!(matches!(targets[0], ShadowTarget::Position(_)));
    }
}
