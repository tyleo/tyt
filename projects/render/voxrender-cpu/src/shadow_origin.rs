use crate::RenderHit;
use ty_math::TyVector3F64;
use voxrender::{GRID_FRACTION_BITS, RenderScene};

/// One cell's side in fixed point.
const ONE: i64 = 1 << GRID_FRACTION_BITS;

/// The fixed point on `hit`'s face `across` units from its min corner along
/// its `u` and `v`, in the hit's grid and in world space.
pub fn shadow_origin(
    scene: &RenderScene,
    hit: &RenderHit,
    across: [i64; 2],
) -> ([i32; 3], TyVector3F64) {
    let face = &hit.face;
    let mut point = [0; 3];

    point[face.d] = (i64::from(face.s) + i64::from(face.sign > 0)) * ONE;
    point[face.u()] = face.u0 as i64 * ONE + across[0];
    point[face.v()] = face.v0 as i64 * ONE + across[1];

    let origin =
        point.map(|coordinate| i32::try_from(coordinate).expect("a grid's sides are within range"));

    let world = scene
        .placement(hit.placement_id)
        .expect("a hit lands on one of the scene's placements")
        .transform
        .transform_point(TyVector3F64::from_array(
            origin.map(|coordinate| coordinate as f64 / ONE as f64),
        ));

    (origin, world)
}
