use crate::{RenderHit, ShadowTarget, placement_bounds, shadow_origin};
use voxrender::{
    Error, RenderGridRay, RenderScene, Result, grid_point, grid_vector, quantize_direction,
    quantize_point,
};

/// One ray per placement of `scene`, in placement order, toward `target`
/// from the fixed point on `hit`'s face `across` units from its min corner
/// along its `u` and `v`. Other grids start the ray at the same point in
/// world space. Errors if a start or the light lies out of a grid's
/// [range](voxrender::GRID_RANGE_BITS).
pub fn shadow_rays(
    scene: &RenderScene,
    hit: &RenderHit,
    across: [i64; 2],
    target: &ShadowTarget,
) -> Result<Vec<RenderGridRay>> {
    let (origin, world) = shadow_origin(scene, hit, across);

    scene
        .iter_placements()
        .map(|(other_id, other)| {
            let bounds = placement_bounds(scene, other);

            let quantize = |point| {
                quantize_point(grid_point(&other.transform, point), bounds).ok_or(
                    Error::GridRange {
                        placement_id: other_id,
                    },
                )
            };

            let start = if other_id == hit.placement_id {
                origin
            } else {
                quantize(world)?
            };

            Ok(match *target {
                ShadowTarget::Direction(direction) => RenderGridRay {
                    origin: start,
                    direction: quantize_direction(grid_vector(&other.transform, direction)),
                    end: None,
                },

                ShadowTarget::Position(position) => {
                    RenderGridRay::toward(start, quantize(position)?)
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{ShadowTarget, cast_ray, shadow_rays};
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxrender::{GRID_FRACTION_BITS, RenderGridRay, RenderRay, fixtures::cells_scene};

    const ONE: i32 = 1 << GRID_FRACTION_BITS;

    #[test]
    fn a_shadow_ray_starts_on_the_face_and_at_the_same_world_point_in_other_grids() {
        let beside = TyTransformF64::new(
            TyVector3F64::new(10.0, 0.0, 0.0),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::splat(2.0),
        );
        let scene = cells_scene(
            [1, 1, 1],
            &[([0, 0, 0], 0)],
            &[TyTransformF64::IDENTITY, beside],
        );

        // The cube's top face, whose u runs along z and v along x.
        let ray = RenderRay {
            origin: TyVector3F64::new(0.5, 4.0, 0.5),
            direction: -TyVector3F64::Y,
        };
        let hit = cast_ray(&scene, &ray, f64::INFINITY).unwrap().unwrap();
        let across = [i64::from(ONE / 4), i64::from(ONE / 2)];

        let up = shadow_rays(
            &scene,
            &hit,
            across,
            &ShadowTarget::Direction(TyVector3F64::Y),
        )
        .unwrap();

        assert_eq!(up[0].origin, [ONE / 2, ONE, ONE / 4]);
        assert_eq!(up[1].origin, [-19 * ONE / 4, ONE / 2, ONE / 8]);
        assert_eq!(up[0].direction, [0, 1 << 16, 0]);
        assert_eq!(up[1].end, None);

        let light = TyVector3F64::new(0.5, 3.0, 0.25);
        let toward = shadow_rays(&scene, &hit, across, &ShadowTarget::Position(light)).unwrap();

        assert_eq!(
            toward[0],
            RenderGridRay::toward([ONE / 2, ONE, ONE / 4], [ONE / 2, 3 * ONE, ONE / 4])
        );
    }
}
