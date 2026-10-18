use crate::{RenderHit, is_opaque, placement_bounds};
use branded_id::U32Id;
use std::{
    array,
    cmp::Ordering,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use ty_math::TyVector3U32;
use voxrender::{
    BRenderMaterial, BRenderPlacement, Error, GRID_FRACTION_BITS, RenderGridRay, RenderObject,
    RenderPlacement, RenderRay, RenderScene, Result,
};
use voxsurface::SurfaceSpan;

/// One cell's side in fixed point.
const ONE: i64 = 1 << GRID_FRACTION_BITS;

/// The surface hits a ray meets across every placement of a scene, nearest
/// first. Each placement walks its grid on its own, remembering the
/// material of the cell the ray is inside: nothing before the ray enters
/// the grid and nothing after it enters an empty cell. Entering a live cell
/// whose material differs is an entry through the face the ray came in by.
/// Leaving a material for an empty cell, the outside of the grid, or a
/// material that is not opaque is an exit through the face the ray leaves
/// by. An exit comes before the entry at the same face. Cells of one
/// material have no seam between them, so a slab of one material shows its
/// near and far faces. A ray that starts inside a cell leaves that cell's
/// material without an exit.
///
/// Each placement walks its [`RenderGridRay`] with an integer DDA. Ties step
/// x, then y, then z. A point on a boundary belongs to the cell the ray
/// moves into.
#[derive(Clone, Debug)]
pub struct RenderRayWalk<'a> {
    walks: Vec<PlacementWalk<'a>>,

    inside: Vec<U32Id<BRenderMaterial>>,
}

impl<'a> RenderRayWalk<'a> {
    /// The walk through `scene` of `rays`, one per placement in the order of
    /// [`RenderScene::iter_placements`]. Panics unless `rays` holds one ray
    /// per placement, each [in range](RenderGridRay::in_range) of its grid.
    pub fn new(scene: &'a RenderScene, rays: impl IntoIterator<Item = RenderGridRay>) -> Self {
        let mut rays = rays.into_iter();

        let walks: Vec<PlacementWalk> = scene
            .iter_placements()
            .filter_map(|(placement_id, placement)| {
                let ray = rays.next().expect("one ray per placement");

                let object = scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's");

                assert!(ray.in_range(object.bounds()), "each ray is within range");

                PlacementWalk::new(scene, placement_id, placement, object, &ray)
            })
            .collect();

        assert!(rays.next().is_none(), "one ray per placement");

        let inside = walks.iter().filter_map(|walk| walk.inside).collect();

        RenderRayWalk { walks, inside }
    }

    /// The walk of the world-space `ray` through `scene`, carried into each
    /// placement's grid. Errors if the ray's origin lies out of a grid's
    /// [range](crate::GRID_RANGE_BITS).
    pub fn from_ray(scene: &'a RenderScene, ray: &RenderRay) -> Result<Self> {
        let rays = scene
            .iter_placements()
            .map(|(placement_id, placement)| {
                RenderGridRay::from_ray(
                    &placement.transform,
                    placement_bounds(scene, placement),
                    ray,
                )
                .ok_or(Error::GridRange { placement_id })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(RenderRayWalk::new(scene, rays))
    }

    /// The materials the ray starts inside, one per placement whose start
    /// cell holds one.
    pub fn starts_inside(&self) -> &[U32Id<BRenderMaterial>] {
        &self.inside
    }
}

/// Each placement advances only as far as the nearest hit another placement
/// has already found, so a placement behind a near hit never walks past it.
/// Hits at one distance come in placement order.
impl Iterator for RenderRayWalk<'_> {
    type Item = RenderHit;

    fn next(&mut self) -> Option<RenderHit> {
        let mut nearest: Option<(usize, f64)> = None;

        for (index, walk) in self.walks.iter_mut().enumerate() {
            let limit = nearest.map_or(f64::INFINITY, |(_, distance)| distance);

            if walk.peeked.is_none() {
                walk.peeked = walk.advance(limit);
            }

            if let Some(hit) = walk.peeked
                && nearest.is_none_or(|(_, distance)| hit.distance < distance)
            {
                nearest = Some((index, hit.distance));
            }
        }

        let (index, _) = nearest?;

        self.walks[index].peeked.take()
    }
}

/// One placement's walk. It mirrors each axis the ray moves down so it only
/// steps up, and every position below is mirrored.
#[derive(Clone)]
struct PlacementWalk<'a> {
    scene: &'a RenderScene,

    placement_id: U32Id<BRenderPlacement>,

    object: &'a RenderObject,

    bounds: [i64; 3],

    /// Which axes are mirrored.
    flip: [bool; 3],

    origin: [i64; 3],

    /// The direction's magnitudes.
    direction: [i64; 3],

    /// Meters per unit of the direction.
    meters: f64,

    /// The last cell on each axis the walk visits.
    last: [i64; 3],

    /// Whether the step past `last` on each axis leaves the grid rather than
    /// passing the ray's end.
    leaves: [bool; 3],

    cell: [i64; 3],

    /// The error terms of the axis pairs xy, xz, and yz. The term of axes
    /// `a < b` is how much later the ray reaches its next boundary on `a`
    /// than on `b`, times both directions.
    error: [i64; 3],

    /// The material of the cell the ray is inside.
    inside: Option<U32Id<BRenderMaterial>>,

    /// Whether the ray is still inside the material it started in, which it
    /// leaves without an exit.
    in_start: bool,

    /// Whether the walk has left the grid or passed its end.
    done: bool,

    /// The next hit, found but not yet yielded.
    peeked: Option<RenderHit>,

    /// The entry at the face of the exit the walk last found.
    queued: Option<RenderHit>,
}

impl<'a> PlacementWalk<'a> {
    /// The walk of `ray` through `object` under `placement`, or `None` when
    /// the ray misses the grid.
    fn new(
        scene: &'a RenderScene,
        placement_id: U32Id<BRenderPlacement>,
        placement: &RenderPlacement,
        object: &'a RenderObject,
        ray: &RenderGridRay,
    ) -> Option<Self> {
        let bounds = object.bounds().to_array().map(i64::from);
        let flip = ray.direction.map(|component| component < 0);

        let origin: [i64; 3] = array::from_fn(|a| {
            let origin = i64::from(ray.origin[a]);

            if flip[a] {
                bounds[a] * ONE - origin
            } else {
                origin
            }
        });

        let direction = ray.direction.map(|component| i64::from(component).abs());

        let edge = |a: usize| bounds[a] - 1;

        // A ray without an end runs out past the grid's edge.
        let end: [i64; 3] = array::from_fn(|a| {
            let Some(end) = ray.end else {
                return edge(a) + 1;
            };

            let end = i64::from(end[a]);

            if flip[a] { edge(a) - end } else { end }
        });

        let last = array::from_fn(|a| edge(a).min(end[a]));
        let leaves = array::from_fn(|a| end[a] > edge(a));

        // On an axis the ray does not move along, the origin must lie within
        // the grid.
        for a in 0..3 {
            if direction[a] == 0 && !(0..bounds[a] * ONE).contains(&origin[a]) {
                return None;
            }
        }

        // A ray from outside enters through the lower face it reaches last.
        // At a tie the higher axis is crossed last.
        let entry = (0..3).filter(|&a| direction[a] > 0 && origin[a] < 0).fold(
            None,
            |entry: Option<usize>, a| match entry {
                Some(e) if -origin[e] * direction[a] > -origin[a] * direction[e] => Some(e),

                _ => Some(a),
            },
        );

        let cell: [i64; 3] = array::from_fn(|a| match entry {
            // The walk stands just outside the face it enters through.
            Some(e) if a == e => -1,

            // `scaled / span` is the ray's coordinate where it reaches the
            // grid's face on `e`. A crossing there on a later axis comes after
            // the entry.
            Some(e) if direction[a] > 0 => {
                let run = direction[e];
                let scaled = origin[a] * run - origin[e] * direction[a];
                let span = run * ONE;
                let cell = scaled.div_euclid(span);

                if a > e && scaled.rem_euclid(span) == 0 {
                    cell - 1
                } else {
                    cell
                }
            }

            _ => origin[a].div_euclid(ONE),
        });

        let within = |a: usize| (0..bounds[a]).contains(&cell[a]);

        if !(0..3).all(|a| entry == Some(a) || within(a)) {
            return None;
        }

        let inside = match entry {
            Some(_) => None,

            None => object.voxel_material(
                object
                    .voxel_id(position_of(cell, flip, bounds))
                    .expect("the start cell is within the grid"),
            ),
        };

        // A ray that goes nowhere only starts inside its cell.
        let done = direction == [0; 3] || (0..3).any(|a| entry != Some(a) && cell[a] > last[a]);

        // The fixed-point distance from the origin to each axis' next
        // boundary.
        let next = |a: usize| (cell[a] + 1) * ONE - origin[a];

        let error_of = |a: usize, b: usize| next(a) * direction[b] - next(b) * direction[a];

        let scale = placement.transform.scale;
        let meters = (0..3)
            .map(|a| (scale[a] * f64::from(ray.direction[a])).powi(2))
            .sum::<f64>()
            .sqrt()
            / ONE as f64;

        Some(PlacementWalk {
            scene,
            placement_id,
            object,
            bounds,
            flip,
            origin,
            direction,
            meters,
            last,
            leaves,
            cell,
            error: [error_of(0, 1), error_of(0, 2), error_of(1, 2)],
            inside,
            in_start: inside.is_some(),
            done,
            peeked: None,
            queued: None,
        })
    }

    /// The next hit within `limit` meters, or `None` when the ray has left
    /// the grid, passed its end, or reaches its next cell past `limit`. A
    /// walk stopped at `limit` stays where it is and resumes on the next
    /// call. An entry queued behind an exit comes first, whatever the limit.
    fn advance(&mut self, limit: f64) -> Option<RenderHit> {
        if let Some(hit) = self.queued.take() {
            return Some(hit);
        }

        if self.done {
            return None;
        }

        loop {
            let axis = self.next_axis();

            let ahead = (self.cell[axis] + 1) * ONE - self.origin[axis];
            let distance = ahead as f64 / self.direction[axis] as f64 * self.meters;

            if distance > limit {
                return None;
            }

            let across = self.across(axis);

            self.cell[axis] += 1;

            if self.cell[axis] > self.last[axis] {
                self.done = true;

                return if self.leaves[axis] {
                    self.exit(axis, distance, across)
                } else {
                    None
                };
            }

            self.step(axis);

            if let Some(hit) = self.cross(axis, distance, across) {
                return Some(hit);
            }
        }
    }

    /// The axis whose next boundary the ray reaches first, ties to the
    /// lowest.
    fn next_axis(&self) -> usize {
        let first = if self.error[0] > 0 { 1 } else { 0 };

        if self.error[pair(first, 2)] > 0 {
            2
        } else {
            first
        }
    }

    /// Moves the error terms past a step along `axis`.
    fn step(&mut self, axis: usize) {
        for other in (0..3).filter(|&other| other != axis) {
            let change = ONE * self.direction[other];

            if axis < other {
                self.error[pair(axis, other)] += change;
            } else {
                self.error[pair(other, axis)] -= change;
            }
        }
    }

    /// The ray's unmirrored offset into its cell on each other axis where it
    /// crosses the boundary ahead on `axis`, times `direction[axis]`.
    fn across(&self, axis: usize) -> [i64; 3] {
        let span = ONE * self.direction[axis];

        array::from_fn(|other| {
            let mirrored = match other.cmp(&axis) {
                Ordering::Equal => return 0,
                Ordering::Greater => span + self.error[pair(axis, other)],
                Ordering::Less => span - self.error[pair(other, axis)],
            };

            if self.flip[other] {
                span - mirrored
            } else {
                mirrored
            }
        })
    }

    /// Crosses into the current cell through its face on `axis` at
    /// `distance`. An exit comes out before the entry it queues.
    fn cross(&mut self, axis: usize, distance: f64, across: [i64; 3]) -> Option<RenderHit> {
        let voxel_id = self
            .object
            .voxel_id(position_of(self.cell, self.flip, self.bounds))
            .expect("the walk stays within the grid");

        let material_id = self.object.voxel_material(voxel_id);

        if material_id == self.inside {
            return None;
        }

        let into_opaque = material_id.is_some_and(|material_id| {
            is_opaque(
                self.scene
                    .material(material_id)
                    .expect("a voxel samples one of the scene's materials"),
            )
        });

        // Leaving for an opaque cell is no exit. The cell's entry lands on
        // the same face.
        let exit = if into_opaque {
            None
        } else {
            self.exit(axis, distance, across)
        };

        self.inside = material_id;
        self.in_start = false;

        let entry = material_id.map(|_| self.hit(self.cell, axis, false, distance, across));

        match exit {
            Some(exit) => {
                self.queued = entry;

                Some(exit)
            }

            None => entry,
        }
    }

    /// The exit through the face behind the current cell on `axis`, if the
    /// ray makes one there.
    fn exit(&self, axis: usize, distance: f64, across: [i64; 3]) -> Option<RenderHit> {
        if self.inside.is_none() || self.in_start {
            return None;
        }

        let mut left = self.cell;
        left[axis] -= 1;

        Some(self.hit(left, axis, true, distance, across))
    }

    /// The hit at `distance` through the face on `axis` of the mirrored
    /// `cell`: the face the ray enters by, or leaves by when `exit`.
    fn hit(
        &self,
        cell: [i64; 3],
        axis: usize,
        exit: bool,
        distance: f64,
        across: [i64; 3],
    ) -> RenderHit {
        let position = position_of(cell, self.flip, self.bounds);

        let voxel_id = self
            .object
            .voxel_id(position)
            .expect("the walk stays within the grid");

        // The ray moves up each mirrored axis, so it enters a cell by its
        // lower face and leaves by its upper one.
        let upper = exit != self.flip[axis];

        let face = SurfaceSpan {
            d: axis,
            sign: if upper { 1 } else { -1 },
            s: position[axis],
            u0: position[(axis + 1) % 3] as usize,
            u1: position[(axis + 1) % 3] as usize + 1,
            v0: position[(axis + 2) % 3] as usize,
            v1: position[(axis + 2) % 3] as usize + 1,
        };

        let run = self.direction[axis];
        let span = ONE * run;

        let along = [face.u(), face.v()].map(|tangent| across[tangent] as f64 / span as f64);

        let point = array::from_fn(|a| {
            let corner = i64::from(position[a]) * ONE;

            let point = if a == axis {
                corner + if upper { ONE } else { 0 }
            } else {
                corner + (2 * across[a] + run) / (2 * run)
            };

            i32::try_from(point).expect("a grid's sides are within range")
        });

        RenderHit {
            placement_id: self.placement_id,
            voxel_id,
            face,
            along,
            point,
            distance,
            exit,
        }
    }
}

/// The scene has no `Debug`, so the walk shows where it stands.
impl Debug for PlacementWalk<'_> {
    fn fmt(&self, formatter: &mut Formatter) -> FmtResult {
        formatter
            .debug_struct("PlacementWalk")
            .field("placement_id", &self.placement_id)
            .field("cell", &self.cell)
            .field("inside", &self.inside)
            .field("done", &self.done)
            .finish_non_exhaustive()
    }
}

/// The index in a walk's error terms of axes `a < b`.
fn pair(a: usize, b: usize) -> usize {
    a + b - 1
}

/// The grid position of the mirrored `cell`, which lies within the grid.
fn position_of(cell: [i64; 3], flip: [bool; 3], bounds: [i64; 3]) -> TyVector3U32 {
    let position: [u32; 3] = array::from_fn(|a| {
        let unmirrored = if flip[a] {
            bounds[a] - 1 - cell[a]
        } else {
            cell[a]
        };

        u32::try_from(unmirrored).expect("the cell lies within the grid")
    });

    TyVector3U32::from_array(position)
}

#[cfg(test)]
mod tests {
    use crate::{RenderRayWalk, ShadowTarget, is_opaque, test_utilities::bar_scene};
    use std::array;
    use ty_math::{TyLinSrgbaF64, TyTransformF64, TyVector3F64, TyVector3U32};
    use voxrender::{
        GRID_FRACTION_BITS, RenderGridRay, RenderLight, RenderRay, RenderScene, RenderViewRays,
        SHADOW_INSET_BITS,
        fixtures::{
            cells_scene, glass, matte, precision_lights, precision_view_scene, random_scene,
            stairs_scene,
        },
        quantize_direction, quantize_point,
    };

    const ONE: i32 = 1 << GRID_FRACTION_BITS;

    fn ray(origin: [f64; 3], direction: [f64; 3]) -> RenderRay {
        RenderRay {
            origin: TyVector3F64::from_array(origin),
            direction: TyVector3F64::from_array(direction).normalize(),
        }
    }

    /// The face layer, the distance, and whether it is an exit, of each hit
    /// along +X from `x`.
    fn layers_along_x(scene: &RenderScene, x: f64) -> Vec<(u32, f64, bool)> {
        RenderRayWalk::from_ray(scene, &ray([x, 0.5, 0.5], [1.0, 0.0, 0.0]))
            .unwrap()
            .map(|hit| {
                assert_eq!(hit.face.d, 0);
                assert_eq!(hit.face.sign, if hit.exit { 1 } else { -1 });

                (hit.face.s, hit.distance, hit.exit)
            })
            .collect()
    }

    #[test]
    fn one_material_shows_its_near_and_far_faces_and_two_their_seam() {
        let one = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 0)],
            &[TyTransformF64::IDENTITY],
        );
        assert_eq!(
            layers_along_x(&one, -1.0),
            [(0, 1.0, false), (1, 3.0, true)]
        );

        // The second material is opaque, so the first has no exit into it.
        let two = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );
        assert_eq!(
            layers_along_x(&two, -1.0),
            [(0, 1.0, false), (1, 2.0, false), (1, 3.0, true)]
        );
    }

    #[test]
    fn glass_exits_into_the_open_and_clear_materials_but_not_into_opaque_ones() {
        let white = TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0);
        let red = TyLinSrgbaF64::new(1.0, 0.0, 0.0, 1.0);

        let hits = |materials: &[_]| {
            let (scene, ray) = bar_scene(materials);

            layers_along_x(&scene, ray.origin.x)
        };

        // A pane, and a slab of one material, exit at the grid's edge.
        assert_eq!(hits(&[glass(white)]), [(0, 1.0, false), (0, 2.0, true)]);
        assert_eq!(
            hits(&[glass(white), glass(white)]),
            [(0, 1.0, false), (1, 3.0, true)]
        );

        // Glass on an opaque wall meets the wall's face alone.
        assert_eq!(
            hits(&[glass(white), matte(red)]),
            [(0, 1.0, false), (1, 2.0, false), (1, 3.0, true)]
        );

        // Between two glasses the exit comes before the entry.
        assert_eq!(
            hits(&[glass(white), glass(red)]),
            [
                (0, 1.0, false),
                (0, 2.0, true),
                (1, 2.0, false),
                (1, 3.0, true)
            ]
        );
    }

    #[test]
    fn a_hollow_box_shows_both_faces_of_its_near_and_far_walls() {
        let shell: Vec<_> = (0..27u32)
            .map(|index| [index / 9, index / 3 % 3, index % 3])
            .filter(|&position| position != [1, 1, 1])
            .map(|position| (position, 0))
            .collect();
        let scene = cells_scene([3, 3, 3], &shell, &[TyTransformF64::IDENTITY]);

        let hits: Vec<_> = RenderRayWalk::from_ray(&scene, &ray([-1.0, 1.5, 1.5], [1.0, 0.0, 0.0]))
            .unwrap()
            .map(|hit| (hit.face.s, hit.distance, hit.exit))
            .collect();
        assert_eq!(
            hits,
            [
                (0, 1.0, false),
                (0, 2.0, true),
                (2, 3.0, false),
                (2, 4.0, true)
            ]
        );
    }

    #[test]
    fn a_ray_from_inside_leaves_its_material_without_an_exit() {
        let scene = cells_scene(
            [3, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 0), ([2, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );

        assert_eq!(
            layers_along_x(&scene, 0.5),
            [(2, 1.5, false), (2, 2.5, true)]
        );
        assert_eq!(
            layers_along_x(&scene, 1.5),
            [(2, 0.5, false), (2, 1.5, true)]
        );
        assert_eq!(layers_along_x(&scene, 2.5), []);

        // Entering the start material again exits it as usual.
        let gap = cells_scene(
            [3, 1, 1],
            &[([0, 0, 0], 0), ([2, 0, 0], 0)],
            &[TyTransformF64::IDENTITY],
        );
        assert_eq!(layers_along_x(&gap, 0.5), [(2, 1.5, false), (2, 2.5, true)]);
    }

    #[test]
    fn the_walk_lists_the_materials_the_ray_starts_inside() {
        let scene = cells_scene(
            [1, 1, 1],
            &[([0, 0, 0], 0)],
            &[
                TyTransformF64::IDENTITY,
                TyTransformF64 {
                    position: TyVector3F64::new(0.25, 0.0, 0.0),
                    ..TyTransformF64::IDENTITY
                },
            ],
        );
        let (material_id, _) = scene.iter_materials().next().unwrap();

        let inside =
            RenderRayWalk::from_ray(&scene, &ray([0.5, 0.5, 0.5], [1.0, 0.0, 0.0])).unwrap();
        assert_eq!(inside.starts_inside(), [material_id, material_id]);

        let outside =
            RenderRayWalk::from_ray(&scene, &ray([-1.0, 0.5, 0.5], [1.0, 0.0, 0.0])).unwrap();
        assert!(outside.starts_inside().is_empty());
    }

    #[test]
    fn placements_interleave_by_distance() {
        // A bar live at both ends, placed at the origin and again two and
        // a half units down -Z, so the hits alternate between the two.
        let scene = cells_scene(
            [1, 1, 5],
            &[([0, 0, 0], 0), ([0, 0, 4], 0)],
            &[
                TyTransformF64::IDENTITY,
                TyTransformF64::from_translation(TyVector3F64::new(0.0, 0.0, -2.5)),
            ],
        );
        let ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();

        let hits: Vec<_> =
            RenderRayWalk::from_ray(&scene, &ray([0.5, 0.5, 10.0], [0.0, 0.0, -1.0]))
                .unwrap()
                .map(|hit| (hit.placement_id, hit.distance))
                .collect();
        assert_eq!(
            hits,
            [
                (ids[0], 5.0),
                (ids[0], 6.0),
                (ids[1], 7.5),
                (ids[1], 8.5),
                (ids[0], 9.0),
                (ids[0], 10.0),
                (ids[1], 11.5),
                (ids[1], 12.5)
            ]
        );
    }

    /// Each hit of `ray` through `scene`'s one placement: the cell, the
    /// face's axis, its sign, and whether it is an exit.
    fn walk_hits(scene: &RenderScene, ray: RenderGridRay) -> Vec<([u32; 3], usize, i32, bool)> {
        let (_, placement) = scene.iter_placements().next().unwrap();
        let object = scene.object(placement.object_id).unwrap();

        RenderRayWalk::new(scene, [ray])
            .map(|hit| {
                let position = object.voxel_position(hit.voxel_id).unwrap().to_array();

                (position, hit.face.d, hit.face.sign, hit.exit)
            })
            .collect()
    }

    /// The hits of an exact rational walk of `ray` from its origin across
    /// the unbounded lattice, through `scene`'s one placement. Adds the
    /// steps that tie to `ties`.
    fn exact_hits(
        scene: &RenderScene,
        ray: RenderGridRay,
        ties: &mut usize,
    ) -> Vec<([u32; 3], usize, i32, bool)> {
        let (_, placement) = scene.iter_placements().next().unwrap();
        let object = scene.object(placement.object_id).unwrap();
        let bounds = object.bounds().to_array().map(i64::from);
        let one = i128::from(ONE);

        let sign = ray.direction.map(|c| if c < 0 { -1 } else { 1 });
        let magnitude = ray.direction.map(|c| i128::from(c).abs());

        // A point on a boundary belongs to the cell the ray moves into.
        let mut cell: [i64; 3] = array::from_fn(|a| {
            let origin = i128::from(ray.origin[a]);

            let cell = if sign[a] < 0 {
                -(-origin).div_euclid(one) - 1
            } else {
                origin.div_euclid(one)
            };

            cell as i64
        });

        // The fixed-point distance from the origin to each axis' next
        // boundary. The ray reaches it at `ahead / magnitude`.
        let mut ahead: [i128; 3] = array::from_fn(|a| {
            let origin = i128::from(ray.origin[a]);
            let corner = i128::from(cell[a]) * one;

            if sign[a] < 0 {
                origin - corner
            } else {
                corner + one - origin
            }
        });

        let in_grid = |cell: [i64; 3]| (0..3).all(|a| (0..bounds[a]).contains(&cell[a]));

        let material = |cell: [i64; 3]| {
            let position = TyVector3U32::from_array(cell.map(|c| c as u32));

            object.voxel_material(object.voxel_id(position).unwrap())
        };

        let past_end = |cell: [i64; 3], a: usize| {
            ray.end
                .is_some_and(|end| sign[a] * (cell[a] - i64::from(end[a])) > 0)
        };

        if (0..3).any(|a| past_end(cell, a)) {
            return Vec::new();
        }

        let opaque = |cell: [i64; 3]| {
            material(cell)
                .is_some_and(|material_id| is_opaque(scene.material(material_id).unwrap()))
        };

        let mut inside = if in_grid(cell) { material(cell) } else { None };
        let mut in_start = inside.is_some();
        let mut hits = Vec::new();

        loop {
            let gone = (0..3).any(|a| match (magnitude[a] == 0, sign[a] > 0) {
                (true, _) => !(0..bounds[a]).contains(&cell[a]),
                (false, true) => cell[a] >= bounds[a],
                (false, false) => cell[a] < 0,
            });

            let active: Vec<usize> = (0..3).filter(|&a| magnitude[a] > 0).collect();

            if gone || active.is_empty() {
                return hits;
            }

            let reach = |a: usize, b: usize| ahead[a] * magnitude[b];

            let axis = active[1..].iter().fold(active[0], |best, &a| {
                if reach(a, best) < reach(best, a) {
                    a
                } else {
                    best
                }
            });

            if active
                .iter()
                .any(|&a| a != axis && reach(a, axis) == reach(axis, a))
            {
                *ties += 1;
            }

            let left = cell;

            cell[axis] += sign[axis];
            ahead[axis] += one;

            if past_end(cell, axis) {
                return hits;
            }

            let entered = if in_grid(cell) { material(cell) } else { None };

            if entered == inside {
                continue;
            }

            if inside.is_some() && !in_start && !(in_grid(cell) && opaque(cell)) {
                hits.push((left.map(|c| c as u32), axis, sign[axis] as i32, true));
            }

            inside = entered;
            in_start = false;

            if entered.is_some() {
                hits.push((cell.map(|c| c as u32), axis, -sign[axis] as i32, false));
            }
        }
    }

    #[test]
    fn corner_shadow_rays_over_the_precision_scenes_match_an_exact_walk() {
        let inset = ONE >> SHADOW_INSET_BITS;
        let mut ties = 0;

        let bounds = TyVector3U32::new(24, 10, 24);

        let targets: Vec<_> = precision_lights()
            .into_iter()
            .map(|light| match light {
                RenderLight::Directional { rotation, .. } => {
                    ShadowTarget::Direction(rotation * TyVector3F64::Z)
                }

                RenderLight::Point { position, .. } => ShadowTarget::Position(position),

                _ => unreachable!("the precision lights are suns and points"),
            })
            .collect();

        for scene in [
            random_scene(1, 0.15, 1),
            random_scene(2, 0.40, 1),
            stairs_scene(),
        ] {
            for x in 0..24 {
                for z in 0..24 {
                    for corner in 0..4 {
                        // A corner of the top face of cell (x, 0, z), inset.
                        let origin = [
                            x * ONE + if corner & 1 == 0 { inset } else { ONE - inset },
                            ONE,
                            z * ONE + if corner & 2 == 0 { inset } else { ONE - inset },
                        ];

                        for target in &targets {
                            let ray = match *target {
                                ShadowTarget::Direction(direction) => RenderGridRay {
                                    origin,
                                    direction: quantize_direction(direction),
                                    end: None,
                                },

                                ShadowTarget::Position(position) => RenderGridRay::toward(
                                    origin,
                                    quantize_point(position, bounds).unwrap(),
                                ),
                            };

                            assert_eq!(
                                walk_hits(&scene, ray),
                                exact_hits(&scene, ray, &mut ties),
                                "{ray:?}"
                            );
                        }
                    }
                }
            }
        }

        assert!(ties > 0);
    }

    #[test]
    fn view_rays_over_the_precision_scene_match_an_exact_walk() {
        let scene = precision_view_scene();
        let (_, placement) = scene.iter_placements().next().unwrap();
        let mut ties = 0;

        for (_, view) in scene.iter_views() {
            let rays = RenderViewRays::new(view, 64, 64)
                .to_grid_rays(&placement.transform, TyVector3U32::new(24, 10, 24))
                .unwrap();

            for y in 0..64 {
                for x in 0..64 {
                    let ray = rays.ray(x, y);

                    assert_eq!(
                        walk_hits(&scene, ray),
                        exact_hits(&scene, ray, &mut ties),
                        "{ray:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_tie_steps_x_then_y_then_z() {
        let half = ONE / 2;

        // From the middle of cell 0 along the diagonal, the ray meets three
        // boundaries at once and crosses them in axis order.
        let scene = cells_scene(
            [2, 2, 2],
            &[([1, 0, 0], 0), ([1, 1, 0], 1), ([1, 1, 1], 2)],
            &[TyTransformF64::IDENTITY],
        );
        let diagonal = RenderGridRay {
            origin: [half; 3],
            direction: [1, 1, 1],
            end: None,
        };
        assert_eq!(
            walk_hits(&scene, diagonal),
            [
                ([1, 0, 0], 0, -1, false),
                ([1, 1, 0], 1, -1, false),
                ([1, 1, 1], 2, -1, false),
                ([1, 1, 1], 0, 1, true)
            ]
        );

        // A ray from outside across the grid's edge crosses x first, so it
        // enters through the y face. It meets the next diagonal cell through
        // its y face too, and so does the mirrored ray.
        let corner = cells_scene(
            [2, 2, 1],
            &[([0, 0, 0], 0), ([1, 1, 0], 0)],
            &[TyTransformF64::IDENTITY],
        );
        let up = RenderGridRay {
            origin: [-half, -half, half],
            direction: [1, 1, 0],
            end: None,
        };
        let down = RenderGridRay {
            origin: [2 * ONE + half, 2 * ONE + half, half],
            direction: [-1, -1, 0],
            end: None,
        };
        assert_eq!(
            walk_hits(&corner, up),
            [
                ([0, 0, 0], 1, -1, false),
                ([0, 0, 0], 0, 1, true),
                ([1, 1, 0], 1, -1, false),
                ([1, 1, 0], 0, 1, true)
            ]
        );
        assert_eq!(
            walk_hits(&corner, down),
            [
                ([1, 1, 0], 1, 1, false),
                ([1, 1, 0], 0, -1, true),
                ([0, 0, 0], 1, 1, false),
                ([0, 0, 0], 0, -1, true)
            ]
        );
    }

    #[test]
    fn a_point_on_a_boundary_belongs_to_the_cell_the_ray_moves_into() {
        let scene = cells_scene(
            [2, 1, 1],
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
            &[TyTransformF64::IDENTITY],
        );
        let materials: Vec<_> = scene.iter_materials().map(|(id, _)| id).collect();
        let half = ONE / 2;

        let starts = |direction| {
            let ray = RenderGridRay {
                origin: [ONE, half, half],
                direction,
                end: None,
            };

            RenderRayWalk::new(&scene, [ray]).starts_inside().to_vec()
        };

        assert_eq!(starts([1, 0, 0]), [materials[1]]);
        assert_eq!(starts([-1, 0, 0]), [materials[0]]);
        assert_eq!(starts([0, 1, 0]), [materials[1]]);
    }

    #[test]
    fn a_ray_toward_a_point_ends_in_its_cell() {
        let scene = cells_scene(
            [1, 1, 5],
            &[([0, 0, 0], 0), ([0, 0, 4], 0)],
            &[TyTransformF64::IDENTITY],
        );
        let half = ONE / 2;
        let toward = |z| RenderGridRay::toward([half, half, 10 * ONE], [half, half, z]);

        let near = [([0, 0, 4], 2, 1, false), ([0, 0, 4], 2, -1, true)];

        assert_eq!(walk_hits(&scene, toward(5 * half)), near);

        // The end on a boundary belongs to the cell the ray moves into. A ray
        // that ends inside a cell meets none of its far faces.
        assert_eq!(
            walk_hits(&scene, toward(ONE)),
            [near[0], near[1], ([0, 0, 0], 2, 1, false)]
        );
        assert_eq!(walk_hits(&scene, toward(ONE + 1)), near);

        // A ray that ends before the grid meets nothing.
        assert_eq!(walk_hits(&scene, toward(6 * ONE)), []);
    }
}
