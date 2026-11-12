use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::TyVector3F64;
use voxrender::{
    BRenderPlacement, Error as RenderError, GRID_FRACTION_BITS, RenderGridRay, RenderScene,
    ShadowTarget, grid_point, grid_vector, quantize_direction, quantize_point,
};
use voxrender_cpu::RenderRayWalk;

/// Whether the light toward `target` reaches the point `origin` of
/// `placement_id`'s grid, in the grid's fixed point. The ray starts in the
/// cell the walk's boundary rule gives the point, as a corner's shadow ray
/// does. Other grids start it at the same point in the world. Errors if the
/// point or the light lies out of a grid's range.
pub fn lattice_light(
    scene: &RenderScene,
    placement_id: U32Id<BRenderPlacement>,
    origin: [i32; 3],
    target: &ShadowTarget,
) -> Result<bool> {
    let one = f64::from(1 << GRID_FRACTION_BITS);

    let placement = scene
        .placement(placement_id)
        .expect("the placement is the scene's");

    let world = placement
        .transform
        .transform_point(TyVector3F64::from_array(
            origin.map(|coordinate| f64::from(coordinate) / one),
        ));

    let rays = scene
        .iter_placements()
        .map(|(other_id, other)| {
            let bounds = scene
                .object(other.object_id)
                .expect("a placement's object is one of the scene's")
                .bounds();

            let quantize = |point| {
                quantize_point(grid_point(&other.transform, point), bounds).ok_or(Error::Render(
                    RenderError::GridRange {
                        placement_id: other_id,
                    },
                ))
            };

            let start = if other_id == placement_id {
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
        .collect::<Result<Vec<_>>>()?;

    let mut walk = RenderRayWalk::new(scene, rays);

    Ok(walk.starts_inside().is_empty() && walk.next().is_none())
}

#[cfg(test)]
mod tests {
    use crate::{bench_scene, lattice_light, reference_hits};
    use ty_math::TyVector3U32;
    use voxrender::{
        GRID_FRACTION_BITS, SHADOW_INSET_BITS, ShadowTarget, grid_vector, quantize_direction,
    };

    /// One cell's side in fixed point.
    const ONE: i32 = 1 << GRID_FRACTION_BITS;

    /// A corner's inset from its face's edges in fixed point.
    const INSET: i32 = ONE >> SHADOW_INSET_BITS;

    #[test]
    fn a_bit_a_lattice_point_gives_the_references_corner_shadows() {
        for name in ["S1", "S5", "S4", "two-placements"] {
            let scene = bench_scene(name).unwrap();
            let placements: Vec<_> = scene.iter_placements().collect();

            let targets: Vec<_> = scene
                .iter_lights()
                .filter_map(|(_, light)| light.shadow_target())
                .collect();

            let (mut samples, mut wrong) = (0, 0);

            for (view_id, _) in scene.iter_views() {
                for hit in reference_hits(&scene, view_id, 48)
                    .unwrap()
                    .into_iter()
                    .flatten()
                {
                    let (placement_id, placement) = placements[hit.placement as usize];
                    let object = scene.object(placement.object_id).unwrap();
                    let bounds = object.bounds();

                    let cell = TyVector3U32::new(
                        hit.voxel / (bounds.y * bounds.z),
                        hit.voxel / bounds.z % bounds.y,
                        hit.voxel % bounds.z,
                    )
                    .as_ivec3();

                    let d = (hit.face / 2) as usize;
                    let sign = if hit.face % 2 == 1 { 1 } else { -1 };
                    let (u, v) = ((d + 1) % 3, (d + 2) % 3);

                    let live = |at: [i32; 3]| {
                        (0..3).all(|a| at[a] >= 0 && at[a] < bounds[a] as i32)
                            && object
                                .voxel_id(TyVector3U32::new(
                                    at[0] as u32,
                                    at[1] as u32,
                                    at[2] as u32,
                                ))
                                .is_some_and(|voxel_id| object.is_live(voxel_id))
                    };

                    for (target, &lit) in targets.iter().zip(&hit.lit) {
                        let ShadowTarget::Direction(toward) = *target else {
                            continue;
                        };

                        let direction =
                            quantize_direction(grid_vector(&placement.transform, toward));

                        // The light is behind the face.
                        if direction[d] * sign <= 0 {
                            continue;
                        }

                        for corner in 0..4u32 {
                            let far = [corner & 1 != 0, corner & 2 != 0];
                            let mut front = cell.to_array();

                            front[d] += sign;

                            // The lattice point on the axes the ray moves
                            // along, and the face's cell on the others.
                            let mut origin = [0; 3];

                            origin[d] = (cell[d] + i32::from(sign > 0)) * ONE;

                            // Whether the ray leaves the face's cell past the
                            // corner on each tangent axis.
                            let mut outward = [false; 2];

                            for (at, axis) in [u, v].into_iter().enumerate() {
                                // On an axis the ray stays still along,
                                // the corner's ray keeps its inset from
                                // the lattice plane, inside the face's
                                // cell.
                                origin[axis] = if direction[axis] != 0 {
                                    (cell[axis] + i32::from(far[at])) * ONE
                                } else if far[at] {
                                    (cell[axis] + 1) * ONE - INSET
                                } else {
                                    cell[axis] * ONE + INSET
                                };

                                outward[at] = if far[at] {
                                    direction[axis] > 0
                                } else {
                                    direction[axis] < 0
                                };
                            }

                            let mut guess =
                                lattice_light(&scene, placement_id, origin, target).unwrap();

                            // A ray that leaves past the corner on both axes
                            // first crosses the cell beside the face on the
                            // axis it moves faster along.
                            if outward[0] && outward[1] {
                                let faster = if direction[u].abs() > direction[v].abs()
                                    || (direction[u].abs() == direction[v].abs() && u < v)
                                {
                                    0
                                } else {
                                    1
                                };

                                let axis = [u, v][faster];
                                let mut beside = front;

                                beside[axis] += if far[faster] { 1 } else { -1 };
                                guess &= !live(beside);
                            }

                            samples += 1;
                            wrong += usize::from(guess != (lit >> corner & 1 == 1));
                        }
                    }
                }
            }

            // S4's sun moves two cells up for each cell along z. Its rays
            // cross two lattice planes at once, and a ray's inset decides
            // which comes first. A point's one bit cannot hold both orders.
            let most = if name == "S4" { samples / 200 } else { 0 };

            assert!(wrong <= most, "{name}: {wrong} of {samples} differ");
        }
    }
}
