use crate::{RenderHit, RenderRayWalk};
use voxrender::{RenderRay, RenderScene, Result};

/// The nearest surface hit of `ray` within `max_distance` across every
/// placement of `scene`: the first hit of its [`RenderRayWalk`]. Errors if
/// the ray's origin lies out of a grid's [range](crate::GRID_RANGE_BITS).
pub fn cast_ray(
    scene: &RenderScene,
    ray: &RenderRay,
    max_distance: f64,
) -> Result<Option<RenderHit>> {
    let hit = RenderRayWalk::from_ray(scene, ray)?.next();

    Ok(hit.filter(|hit| hit.distance <= max_distance))
}

#[cfg(test)]
mod tests {
    use crate::cast_ray;
    use std::f64::consts::PI;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxrender::{RenderRay, fixtures::cells_scene};

    fn ray(origin: [f64; 3], direction: [f64; 3]) -> RenderRay {
        RenderRay {
            origin: TyVector3F64::from_array(origin),
            direction: TyVector3F64::from_array(direction).normalize(),
        }
    }

    #[test]
    fn a_ray_meets_each_face_of_a_cube_where_it_lands() {
        let scene = cells_scene([1, 1, 1], &[([0, 0, 0], 0)], &[TyTransformF64::IDENTITY]);

        // Rays come from three units out along each axis, both ways. The
        // hit's face faces the ray, and the hit lands a quarter along each
        // tangent.
        for axis in 0..3 {
            for sign in [-1i32, 1] {
                let mut origin = [0.25; 3];
                origin[axis] = if sign > 0 { 4.0 } else { -3.0 };
                let mut direction = [0.0; 3];
                direction[axis] = -f64::from(sign);

                let hit = cast_ray(&scene, &ray(origin, direction), f64::INFINITY)
                    .unwrap()
                    .unwrap();

                assert_eq!(hit.face.d, axis);
                assert_eq!(hit.face.sign, sign);
                assert_eq!(hit.face.s, 0);
                assert_eq!(hit.along, [0.25, 0.25]);
                assert_eq!(hit.distance, 3.0);
            }
        }
    }

    #[test]
    fn a_ray_beside_the_cube_misses() {
        let scene = cells_scene([1, 1, 1], &[([0, 0, 0], 0)], &[TyTransformF64::IDENTITY]);

        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 1.5, 4.0], [0.0, 0.0, -1.0]),
                f64::INFINITY
            )
            .unwrap(),
            None
        );
        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 0.5, 4.0], [0.0, 0.0, 1.0]),
                f64::INFINITY
            )
            .unwrap(),
            None
        );
        assert_eq!(
            cast_ray(&scene, &ray([0.5, 0.5, 4.0], [0.0, 0.0, -1.0]), 2.0).unwrap(),
            None
        );
    }

    #[test]
    fn a_ray_from_inside_a_voxel_leaves_its_material_first() {
        let scene = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );

        let hit = cast_ray(
            &scene,
            &ray([0.5, 0.5, 0.5], [1.0, 0.0, 0.0]),
            f64::INFINITY,
        )
        .unwrap()
        .unwrap();
        assert_eq!(hit.face.d, 0);
        assert_eq!(hit.face.sign, -1);
        assert_eq!(hit.face.s, 1);
        assert_eq!(hit.distance, 0.5);

        assert_eq!(
            cast_ray(
                &scene,
                &ray([0.5, 0.5, 0.5], [-1.0, 0.0, 0.0]),
                f64::INFINITY
            )
            .unwrap(),
            None
        );
    }

    #[test]
    fn the_nearer_placement_wins_and_the_transform_applies() {
        let far = TyTransformF64::from_translation(TyVector3F64::new(0.0, 0.0, -5.0));
        let stacked = cells_scene(
            [1, 1, 1],
            &[([0, 0, 0], 0)],
            &[far, TyTransformF64::IDENTITY],
        );

        let hit = cast_ray(
            &stacked,
            &ray([0.5, 0.5, 10.0], [0.0, 0.0, -1.0]),
            f64::INFINITY,
        )
        .unwrap()
        .unwrap();
        assert_eq!(hit.distance, 9.0);
        assert_eq!(
            hit.placement_id,
            stacked.iter_placements().nth(1).unwrap().0
        );

        // Scaled by two and turned a quarter about y at x = 10, the cube
        // spans 10..12 on x, 0..2 on y, and -2..0 on z. A ray down -Z at
        // (11, 1) meets its +z side at z = 0.
        let turned = TyTransformF64::new(
            TyVector3F64::new(10.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, PI / 2.0),
            TyVector3F64::splat(2.0),
        );
        let turned_scene = cells_scene([1, 1, 1], &[([0, 0, 0], 0)], &[turned]);

        let hit = cast_ray(
            &turned_scene,
            &ray([11.0, 1.0, 5.0], [0.0, 0.0, -1.0]),
            f64::INFINITY,
        )
        .unwrap()
        .unwrap();
        assert!((hit.distance - 5.0).abs() < 1e-9);
        // Grid +X turns onto world -Z, so the ray runs along grid +X and
        // meets the grid's -x face.
        assert_eq!(hit.face.d, 0);
        assert_eq!(hit.face.sign, -1);
    }
}
