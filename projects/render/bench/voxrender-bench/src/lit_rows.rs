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
/// reads them: a bit for each ray a face's corner can cast toward the
/// light, set where the light reaches it.
///
/// The contract casts a corner's ray from a point just inside the face's
/// edges. Along an axis the ray moves along, every face that shares the
/// corner's lattice point casts the same cells' ray, so the point holds
/// one bit. Along an axis it stays still along, the ray keeps to its
/// face's side of the lattice plane. Each cell then holds a bit for either
/// edge. A ray that leaves its face's cell past the corner on both of the
/// face's axes also crosses the cell beside the face first, which the
/// shader reads from the tile's opaque cells.
///
/// A tile's bits fill rows of 32 along z. An axis takes a tile's 17
/// lattice points as its index, or twice a cell plus its far edge where
/// the ray stays still.
pub struct LitRows {
    /// Per placement: the first word of its tiles' rows, the rows a tile
    /// holds, the rows a tile holds along y, and a bit per axis the rays
    /// stay still along.
    pub places: Vec<[u32; 4]>,

    /// Per placement: the direction toward the light in its grid, as the
    /// reference's walk quantizes it.
    pub directions: Vec<[i32; 3]>,

    /// Each placement's tiles' rows, a tile after another in the order of
    /// their words.
    pub words: Vec<u32>,
}

impl LitRows {
    /// Casts the rows of `scene` toward a light along the unit `toward`.
    /// Errors if a ray lies out of a grid's range.
    ///
    /// # Arguments
    ///
    /// 1. `corners`: each object's non-empty tiles' low corners, in the
    ///    order of the tiles' words
    /// 2. `opaque`: whether a material id passes no light
    pub fn new(
        scene: &RenderScene,
        toward: TyVector3F64,
        corners: &HashMap<U32Id<BVoxObject>, Vec<TyVector3U32>>,
        opaque: &dyn Fn(u32) -> bool,
    ) -> Result<Self> {
        let target = ShadowTarget::Direction(toward);

        let mut rows = LitRows {
            places: Vec::new(),
            directions: Vec::new(),
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
            let sides = still.map(|still| if still { 2 * TILE } else { TILE + 1 });
            let tile_rows = (sides[0] * sides[1]) as usize;
            let first = rows.words.len();

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
            let tiles = &corners[&placement.object_id];

            rows.places.push([
                u32::try_from(first).expect("a scene's lit rows number under 2^32"),
                tile_rows as u32,
                sides[1],
                u32::from(still[0]) | u32::from(still[1]) << 1 | u32::from(still[2]) << 2,
            ]);
            rows.directions.push(direction);
            rows.words.resize(first + tiles.len() * tile_rows, 0);

            // The rays cast so far, a bit each as `words` holds them.
            let mut cast = vec![0u32; tiles.len() * tile_rows];

            for (ordinal, corner) in tiles.iter().enumerate() {
                let corner = corner.as_ivec3().to_array();

                for index in 0..TILE.pow(3) {
                    let local = [index / (TILE * TILE), index / TILE % TILE, index % TILE]
                        .map(|coordinate| coordinate as i32);
                    let cell = [0, 1, 2].map(|axis| corner[axis] + local[axis]);

                    if value(cell) == 0 {
                        continue;
                    }

                    // Each face the light can reach: the one on each axis
                    // that looks toward it, with no opaque cell before it.
                    for d in (0..3).filter(|&d| direction[d] != 0) {
                        let ahead = direction[d] > 0;
                        let mut front = cell;

                        front[d] += if ahead { 1 } else { -1 };

                        if blocks(front) {
                            continue;
                        }

                        for far in [[false, false], [true, false], [false, true], [true, true]] {
                            let mut key = [0; 3];
                            let mut origin = [0; 3];

                            key[d] = local[d] + i32::from(ahead);
                            origin[d] = (cell[d] + i32::from(ahead)) * ONE;

                            for (far, axis) in far.into_iter().zip([(d + 1) % 3, (d + 2) % 3]) {
                                if still[axis] {
                                    key[axis] = 2 * local[axis] + i32::from(far);
                                    origin[axis] =
                                        cell[axis] * ONE + if far { ONE - INSET } else { INSET };
                                } else {
                                    key[axis] = local[axis] + i32::from(far);
                                    origin[axis] = (cell[axis] + i32::from(far)) * ONE;
                                }
                            }

                            let row = ordinal * tile_rows
                                + (key[0] as u32 * sides[1] + key[1] as u32) as usize;
                            let bit = 1 << key[2];

                            if cast[row] & bit != 0 {
                                continue;
                            }

                            cast[row] |= bit;

                            if lattice_light(scene, placement_id, origin, &target)? {
                                rows.words[first + row] |= bit;
                            }
                        }
                    }
                }
            }
        }

        Ok(rows)
    }
}
