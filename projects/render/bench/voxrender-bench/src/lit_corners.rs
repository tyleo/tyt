use crate::{Result, lattice_light};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{
    GRID_FRACTION_BITS, RenderScene, SHADOW_INSET_BITS, ShadowTarget, grid_vector,
    quantize_direction,
};
use voxrender_wgpu::dense_cells;

/// One cell's side in fixed point.
const ONE: i32 = 1 << GRID_FRACTION_BITS;

/// A corner's inset from its face's edges in fixed point.
const INSET: i32 = ONE >> SHADOW_INSET_BITS;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// A scene's corner shadows under one directional light, as `box_view.wgsl`
/// reads them: a word for each record cell of each placement's tiles, in
/// the order of [`TileMasks`](crate::TileMasks)' records. Face `f`
/// of the cell takes bits `4 * f` to `4 * f + 3`, a bit for each corner in
/// the occlusion words' order, set where the light reaches the corner. A
/// face the light cannot reach holds zero.
///
/// The contract casts a corner's ray from a point just inside the face's
/// edges. Where a placement's rays can cross two lattice planes at once
/// inside its grid, that inset decides which cell a ray crosses first. Each
/// corner there casts a ray. Elsewhere the fill casts fewer rays than
/// corners:
///
/// 1. Along an axis the ray moves along, every face that shares the
///    corner's lattice point casts the same cells' ray, so the point casts
///    one
/// 2. Along an axis the ray stays still along, the ray keeps to its face's
///    side of the lattice plane, so each cell casts one for either edge
/// 3. A ray that leaves its face's cell past the corner on both of the
///    face's axes first crosses the cell beside the face on the axis it
///    moves faster along. An opaque cell there shadows the corner
pub struct LitCorners {
    /// The first word of each placement's tiles' words.
    pub places: Vec<u32>,

    /// Each placement's tiles' words, a tile after another in the order of
    /// their mask words.
    pub words: Vec<u32>,
}

impl LitCorners {
    /// Casts the corners of `scene` toward a light along the unit `toward`.
    /// Errors if a ray lies out of a grid's range.
    ///
    /// # Arguments
    ///
    /// 1. `tiles`: each object's non-empty tiles in the order of their mask
    ///    words, as a tile's low corner and the indices of its record
    ///    cells
    /// 2. `opaque`: whether a material id passes no light
    pub fn new(
        scene: &RenderScene,
        toward: TyVector3F64,
        tiles: &HashMap<U32Id<BVoxObject>, Vec<(TyVector3U32, Vec<u16>)>>,
        opaque: &dyn Fn(u32) -> bool,
    ) -> Result<Self> {
        let target = ShadowTarget::Direction(toward);

        let mut corners = LitCorners {
            places: Vec::new(),
            words: Vec::new(),
        };

        for (placement_id, placement) in scene.iter_placements() {
            let object = scene
                .object(placement.object_id)
                .expect("a placement's object is one of the scene's");

            let bounds = object.bounds();
            let cells = dense_cells(object);
            let direction = quantize_direction(grid_vector(&placement.transform, toward));
            let still = direction.map(|component| component == 0);

            // Whether a ray from a lattice point can meet a lattice edge
            // inside the grid: along two axes it moves along, it crosses
            // whole cells of both at once.
            let exact = [(0, 1), (1, 2), (0, 2)].into_iter().any(|(a, b)| {
                let (along_a, along_b) = (direction[a].unsigned_abs(), direction[b].unsigned_abs());

                along_a != 0 && along_b != 0 && {
                    let shared = shared_factor(along_a, along_b);

                    along_a / shared <= bounds[a] && along_b / shared <= bounds[b]
                }
            });

            // A tile's rays fill rows of bits along z. An axis takes the
            // tile's 17 lattice points as its index, or twice a cell plus
            // its far edge where the rays stay still.
            let sides = still.map(|still| if still { 2 * TILE } else { TILE + 1 });
            let rows = (sides[0] * sides[1]) as usize;

            // The value of the cell at `cell`: 0 when empty or outside the
            // object, else its material id plus one.
            let value = |cell: [i32; 3]| {
                let inside = (0..3).all(|axis| cell[axis] >= 0 && cell[axis] < bounds[axis] as i32);

                if inside {
                    let [x, y, z] = cell.map(|coordinate| coordinate as usize);

                    cells[(x * bounds.y as usize + y) * bounds.z as usize + z]
                } else {
                    0
                }
            };

            let blocks = |cell| value(cell).checked_sub(1).is_some_and(opaque);

            corners.places.push(
                u32::try_from(corners.words.len()).expect("a scene's lit words number under 2^32"),
            );

            // The tile's rays cast so far, and the ones the light reaches.
            let mut cast = vec![0u32; rows];
            let mut reached = vec![0u32; rows];

            for (corner, recorded) in &tiles[&placement.object_id] {
                let corner = corner.as_ivec3().to_array();

                cast.fill(0);
                reached.fill(0);

                for &index in recorded {
                    let index = u32::from(index);
                    let local = [index / TILE % TILE, index / (TILE * TILE), index % TILE]
                        .map(|coordinate| coordinate as i32);
                    let cell = [0, 1, 2].map(|axis| corner[axis] + local[axis]);
                    let mut word = 0;

                    // Each face the light can reach: the one on each axis
                    // that looks toward it, with no opaque cell before it.
                    for d in (0..3).filter(|&d| direction[d] != 0) {
                        let ahead = direction[d] > 0;
                        let (u, v) = ((d + 1) % 3, (d + 2) % 3);
                        let mut front = cell;

                        front[d] += if ahead { 1 } else { -1 };

                        if blocks(front) {
                            continue;
                        }

                        // The ray moves faster along one of the face's
                        // axes. A tie goes to the earlier axis.
                        let faster = if direction[u].abs() > direction[v].abs()
                            || (direction[u].abs() == direction[v].abs() && u < v)
                        {
                            0
                        } else {
                            1
                        };

                        for (at, far) in
                            [[false, false], [true, false], [false, true], [true, true]]
                                .into_iter()
                                .enumerate()
                        {
                            let mut key = [0; 3];
                            let mut origin = [0; 3];

                            key[d] = local[d] + i32::from(ahead);
                            origin[d] = (cell[d] + i32::from(ahead)) * ONE;

                            for (far, axis) in far.into_iter().zip([u, v]) {
                                if exact || still[axis] {
                                    key[axis] = 2 * local[axis] + i32::from(far);
                                    origin[axis] =
                                        cell[axis] * ONE + if far { ONE - INSET } else { INSET };
                                } else {
                                    key[axis] = local[axis] + i32::from(far);
                                    origin[axis] = (cell[axis] + i32::from(far)) * ONE;
                                }
                            }

                            let lit = if exact {
                                lattice_light(scene, placement_id, origin, &target)?
                            } else {
                                let row = (key[0] as u32 * sides[1] + key[1] as u32) as usize;
                                let bit = 1 << key[2];

                                if cast[row] & bit == 0 {
                                    cast[row] |= bit;

                                    if lattice_light(scene, placement_id, origin, &target)? {
                                        reached[row] |= bit;
                                    }
                                }

                                // Whether the ray leaves the face's cell
                                // past the corner on each of the face's
                                // axes.
                                let outward = [0, 1].map(|side| {
                                    let along = direction[[u, v][side]];

                                    if far[side] { along > 0 } else { along < 0 }
                                });

                                let mut beside = front;

                                beside[[u, v][faster]] += if far[faster] { 1 } else { -1 };

                                reached[row] & bit != 0
                                    && !(outward[0] && outward[1] && blocks(beside))
                            };

                            if lit {
                                word |= 1 << (4 * (2 * d + usize::from(ahead)) + at);
                            }
                        }
                    }

                    corners.words.push(word);
                }
            }
        }

        Ok(corners)
    }
}

/// The greatest whole number that divides both `a` and `b`. Neither can be
/// zero.
fn shared_factor(a: u32, b: u32) -> u32 {
    let (mut a, mut b) = (a, b);

    while b != 0 {
        (a, b) = (b, a % b);
    }

    a
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, LitCorners, TileMasks, bench_scenes, reference_hits};
    use std::collections::HashMap;
    use ty_math::TyVector3U32;
    use voxrender::{ShadowTarget, grid_vector, quantize_direction};

    #[test]
    fn the_words_hold_the_references_corner_shadows() {
        let mut samples = 0;

        for (name, scene) in bench_scenes() {
            // The research's scenes and the bush take long to fill.
            if name.starts_with('S') || name == "inside a bush" {
                continue;
            }

            let targets: Vec<_> = scene
                .iter_lights()
                .filter_map(|(_, light)| light.shadow_target())
                .collect();

            let Some((sun, toward)) =
                targets
                    .iter()
                    .enumerate()
                    .find_map(|(at, target)| match *target {
                        ShadowTarget::Direction(toward) => Some((at, toward)),
                        ShadowTarget::Position(_) => None,
                    })
            else {
                continue;
            };

            let every = |_| true;

            let tiles: HashMap<_, _> = scene
                .iter_objects()
                .map(|(object_id, object)| {
                    let grid = CellGrid::new(object, &|id| id);
                    let masks = TileMasks::new(&grid, true, Some(&every)).unwrap();

                    (
                        object_id,
                        masks
                            .corners
                            .into_iter()
                            .zip(masks.recorded)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();

            let lit = LitCorners::new(&scene, toward, &tiles, &every).unwrap();
            let placements: Vec<_> = scene.iter_placements().collect();

            for (view_id, _) in scene.iter_views() {
                for hit in reference_hits(&scene, view_id, 48)
                    .unwrap()
                    .into_iter()
                    .flatten()
                {
                    let (_, placement) = placements[hit.placement as usize];
                    let bounds = scene.object(placement.object_id).unwrap().bounds();

                    let cell = TyVector3U32::new(
                        hit.voxel / (bounds.y * bounds.z),
                        hit.voxel / bounds.z % bounds.y,
                        hit.voxel % bounds.z,
                    );

                    let direction = quantize_direction(grid_vector(&placement.transform, toward));
                    let axis = (hit.face / 2) as usize;
                    let up = hit.face % 2 == 1;

                    // The light is behind the face.
                    if direction[axis] == 0 || (direction[axis] > 0) != up {
                        continue;
                    }

                    // The cell's word, by its place among the record cells
                    // of its object's tiles.
                    let tiles = &tiles[&placement.object_id];
                    let at = cell % 16;
                    let index = ((at.y * 16 + at.x) * 16 + at.z) as u16;

                    let ordinal = tiles
                        .iter()
                        .position(|&(corner, _)| corner == cell / 16 * 16)
                        .unwrap();

                    let before: usize = tiles[..ordinal]
                        .iter()
                        .map(|(_, recorded)| recorded.len())
                        .sum();

                    let rank = tiles[ordinal]
                        .1
                        .iter()
                        .position(|&held| held == index)
                        .unwrap();

                    let word =
                        lit.words[lit.places[hit.placement as usize] as usize + before + rank];

                    assert_eq!(
                        word >> (4 * hit.face) & 15,
                        u32::from(hit.lit[sun]),
                        "{name}, placement {}, cell {cell}, face {}",
                        hit.placement,
                        hit.face
                    );

                    samples += 1;
                }
            }
        }

        assert!(samples > 1000, "{samples} faces compared");
    }
}
