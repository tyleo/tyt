use crate::{Error, Result, live_boxes};
use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The cells of a tile.
const CELLS: usize = (TILE * TILE * TILE) as usize;

/// The most solid boxes a tile gives up.
const SOLIDS: usize = 4;

/// The fewest cells a solid box holds.
const SOLID_CELLS: u32 = 32;

/// A box of cells: its low cell and the corner past its high cell.
type CellBox = (TyVector3U32, TyVector3U32);

/// An object's non-empty tiles as a mask march reads them.
///
/// A tile's words start with a bit per cell. Cell `(x, y, z)` has index
/// `(x * 16 + y) * 16 + z` and is live where bit `index % 32` of word
/// `index / 32` is set. With materials, the tile's words go on:
///
/// 1. One word of `size | value << 4`. `size` is the bits of a cell's entry,
///    0, 1, 2, 4, or 8. At 0, every live cell holds the material id `value`
/// 2. `128 * size` words of entries. Cell `index`'s entry sits at bit
///    `index * size % 32` of word `index * size / 32`
/// 3. The material ids that the entries index
pub struct TileMasks {
    /// Boxes that hold only live cells. A raster draws their faces as the
    /// cells' faces, with no march.
    pub solids: Vec<CellBox>,

    /// The offset in [`words`](Self::words) of each solid box's tile.
    pub solid_tiles: Vec<u32>,

    /// The box around each tile's live cells outside its solid boxes, in
    /// raster order.
    pub boxes: Vec<CellBox>,

    /// The offset in [`words`](Self::words) of each box's tile.
    pub tiles: Vec<u32>,

    /// Each non-empty tile's words.
    pub words: Vec<u32>,
}

impl TileMasks {
    /// The words of one tile's bits.
    const BIT_WORDS: usize = CELLS / 32;

    /// The masks of `object`'s non-empty tiles, with each tile's materials
    /// when `materials`. Errors if a tile holds over 256 materials.
    ///
    /// When `solids`, each tile first gives up to four solid boxes. Each is
    /// the largest slab of live cells alone off a face of the box around
    /// the tile's other live cells, as long as one holds 32 cells or more.
    /// A slab stays in the march when the cells left would cover over half
    /// of the face it shows them.
    pub fn new(object: &RenderObject, solids: bool, materials: bool) -> Result<Self> {
        let bounds = object.bounds();
        let cells = dense_cells(object);

        let mut masks = TileMasks {
            solids: Vec::new(),
            solid_tiles: Vec::new(),
            boxes: Vec::new(),
            tiles: Vec::new(),
            words: Vec::new(),
        };

        for (low, _) in live_boxes(object, TILE) {
            let corner = low / TILE * TILE;

            // Each cell's value: 0 when empty, else its material id plus
            // one.
            let values: Vec<u32> = (0..CELLS)
                .map(|index| {
                    let cell = corner + position(index);

                    if cell.cmplt(bounds).all() {
                        cells[((cell.x * bounds.y + cell.y) * bounds.z + cell.z) as usize]
                    } else {
                        0
                    }
                })
                .collect();

            let live: Vec<bool> = values.iter().map(|&value| value != 0).collect();
            let offset =
                u32::try_from(masks.words.len()).expect("an object's mask words number under 2^32");

            masks.words.resize(masks.words.len() + Self::BIT_WORDS, 0);

            for index in (0..CELLS).filter(|&index| live[index]) {
                masks.words[offset as usize + index / 32] |= 1 << (index % 32);
            }

            if materials {
                push_materials(&mut masks.words, &values)?;
            }

            let mut tile = Tile {
                left: live.clone(),
                live,
            };

            if solids {
                for _ in 0..SOLIDS {
                    let Some((low, high)) = tile.around().and_then(|around| tile.slab(around))
                    else {
                        break;
                    };

                    masks.solids.push((corner + low, corner + high));
                    masks.solid_tiles.push(offset);
                }
            }

            if let Some((low, high)) = tile.around() {
                masks.boxes.push((corner + low, corner + high));
                masks.tiles.push(offset);
            }
        }

        Ok(masks)
    }
}

/// Appends a tile's materials to `words`, from each cell's `values`: 0 when
/// empty, else its material id plus one. Errors if the tile holds over 256
/// materials.
fn push_materials(words: &mut Vec<u32>, values: &[u32]) -> Result<()> {
    let mut palette: Vec<u32> = Vec::new();

    let entries: Vec<usize> = values
        .iter()
        .map(|&value| match value.checked_sub(1) {
            Some(id) => palette
                .iter()
                .position(|&held| held == id)
                .unwrap_or_else(|| {
                    palette.push(id);

                    palette.len() - 1
                }),

            None => 0,
        })
        .collect();

    let size = match palette.len() {
        0 | 1 => 0,
        2 => 1,
        3..=4 => 2,
        5..=16 => 4,
        17..=256 => 8,
        materials => return Err(Error::TileMaterials { materials }),
    };

    if size == 0 {
        words.push(palette.first().copied().unwrap_or(0) << 4);

        return Ok(());
    }

    words.push(size as u32);

    let first = words.len();

    words.resize(first + CELLS * size / 32, 0);

    for (index, entry) in entries.into_iter().enumerate() {
        words[first + index * size / 32] |= (entry as u32) << (index * size % 32);
    }

    words.extend(palette);

    Ok(())
}

/// One tile's cells while its solid boxes come off. A cell's index is
/// `(x * 16 + y) * 16 + z`.
struct Tile {
    /// Whether each cell is live.
    live: Vec<bool>,

    /// Whether each cell is live and outside every solid box so far.
    left: Vec<bool>,
}

impl Tile {
    /// The box around the cells left, or `None` when no cell is left.
    fn around(&self) -> Option<CellBox> {
        let mut low = TyVector3U32::MAX;
        let mut high = TyVector3U32::ZERO;

        for index in (0..CELLS).filter(|&index| self.left[index]) {
            low = low.min(position(index));
            high = high.max(position(index) + 1);
        }

        (low.x < high.x).then_some((low, high))
    }

    /// Takes the largest slab of live cells alone off a face of `around`,
    /// the box around the cells left, and returns it. A slab spans the box
    /// across its face. Takes nothing and returns `None` when the largest
    /// slab holds under [`SOLID_CELLS`] cells, or when the cells left
    /// behind it would cover over half of the face it shows them.
    fn slab(&mut self, (low, high): CellBox) -> Option<CellBox> {
        let extent = high - low;
        let mut best = None;
        let mut most = SOLID_CELLS - 1;

        for axis in 0..3 {
            let across = extent.x * extent.y * extent.z / extent[axis];

            for from_low in [true, false] {
                // The layers from the face inward that are live throughout.
                let depth = (0..extent[axis])
                    .take_while(|&layer| {
                        let mut layer_low = low;
                        layer_low[axis] = if from_low {
                            low[axis] + layer
                        } else {
                            high[axis] - 1 - layer
                        };

                        let mut layer_high = high;
                        layer_high[axis] = layer_low[axis] + 1;

                        self.all_live(layer_low, layer_high)
                    })
                    .count() as u32;

                if depth * across > most {
                    let mut slab = (low, high);

                    if from_low {
                        slab.1[axis] = low[axis] + depth;
                    } else {
                        slab.0[axis] = high[axis] - depth;
                    }

                    most = depth * across;
                    best = Some((slab, axis, across));
                }
            }
        }

        let ((slab_low, slab_high), axis, across) = best?;
        let left = self.left.clone();

        self.remove(slab_low, slab_high);

        // The cells left cover the slab's inner face where their box meets
        // it.
        let covered = self.around().map_or(0, |(rest_low, rest_high)| {
            let rest = rest_high - rest_low;

            rest.x * rest.y * rest.z / rest[axis]
        });

        if covered * 2 > across {
            self.left = left;

            return None;
        }

        Some((slab_low, slab_high))
    }

    /// Whether every cell from `low` to before `high` is live.
    fn all_live(&self, low: TyVector3U32, high: TyVector3U32) -> bool {
        (low.x..high.x).all(|x| {
            (low.y..high.y)
                .all(|y| (low.z..high.z).all(|z| self.live[((x * TILE + y) * TILE + z) as usize]))
        })
    }

    /// Takes the cells from `low` to before `high` out of the cells left.
    fn remove(&mut self, low: TyVector3U32, high: TyVector3U32) {
        for x in low.x..high.x {
            for y in low.y..high.y {
                for z in low.z..high.z {
                    self.left[((x * TILE + y) * TILE + z) as usize] = false;
                }
            }
        }
    }
}

/// The cell of `index` in a tile.
fn position(index: usize) -> TyVector3U32 {
    let index = index as u32;

    TyVector3U32::new(index / (TILE * TILE), index / TILE % TILE, index % TILE)
}

#[cfg(test)]
mod tests {
    use crate::{TileMasks, b1_scenes, painted_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxrender::RenderObject;

    /// The material id a tile's words hold for the cell of index `bit`, as
    /// the shader reads it.
    fn material(words: &[u32], tile: u32, bit: u32) -> u32 {
        let header = words[tile as usize + 128];
        let size = header & 15;

        if size == 0 {
            return header >> 4;
        }

        let at = bit * size;
        let entries = tile + 129;
        let entry = words[(entries + at / 32) as usize] >> (at % 32) & ((1 << size) - 1);

        words[(entries + 128 * size + entry) as usize]
    }

    /// Whether `cell` of `object` is live.
    fn is_live(object: &RenderObject, cell: TyVector3U32) -> bool {
        object
            .voxel_id(cell)
            .is_some_and(|voxel_id| object.is_live(voxel_id))
    }

    #[test]
    fn the_solids_and_masks_hold_every_live_cell_of_each_scene() {
        for (name, scene) in b1_scenes() {
            for (object_id, object) in scene.iter_objects() {
                for (solids, materials) in [(false, false), (true, false), (true, true)] {
                    let masks = TileMasks::new(object, solids, materials).unwrap();
                    let bounds = object.bounds();
                    let mut held = vec![false; (bounds.x * bounds.y * bounds.z) as usize];

                    assert!(solids || masks.solids.is_empty());

                    for &(low, high) in &masks.solids {
                        for x in low.x..high.x {
                            for y in low.y..high.y {
                                for z in low.z..high.z {
                                    let cell = TyVector3U32::new(x, y, z);

                                    assert!(
                                        is_live(object, cell),
                                        "{name}, object {object_id}, solid cell {cell}"
                                    );

                                    held[((x * bounds.y + y) * bounds.z + z) as usize] = true;
                                }
                            }
                        }
                    }

                    for (&(low, high), &tile) in masks.boxes.iter().zip(&masks.tiles) {
                        let corner = low / 16 * 16;

                        for bit in 0..4096 {
                            let at = TyVector3U32::new(bit / 256, bit / 16 % 16, bit % 16);
                            let word = masks.words[(tile + bit / 32) as usize];
                            let set = word >> (bit % 32) & 1 == 1;
                            let cell = corner + at;

                            assert_eq!(
                                set,
                                is_live(object, cell),
                                "{name}, object {object_id}, cell {cell}"
                            );

                            if set && cell.cmpge(low).all() && cell.cmplt(high).all() {
                                held[((cell.x * bounds.y + cell.y) * bounds.z + cell.z) as usize] =
                                    true;
                            }
                        }
                    }

                    if materials {
                        let tiles = masks.tiles.iter().zip(&masks.boxes);
                        let solid_tiles = masks.solid_tiles.iter().zip(&masks.solids);

                        for (&tile, &(low, high)) in tiles.chain(solid_tiles) {
                            let corner = low / 16 * 16;

                            for x in low.x..high.x {
                                for y in low.y..high.y {
                                    for z in low.z..high.z {
                                        let cell = TyVector3U32::new(x, y, z);
                                        let at = cell - corner;
                                        let bit = (at.x * 16 + at.y) * 16 + at.z;

                                        if !is_live(object, cell) {
                                            continue;
                                        }

                                        assert_eq!(
                                            Some(U32Id::from_u32(material(
                                                &masks.words,
                                                tile,
                                                bit
                                            ))),
                                            object.voxel_material(object.voxel_id(cell).unwrap()),
                                            "{name}, object {object_id}, cell {cell}"
                                        );
                                    }
                                }
                            }
                        }
                    }

                    for index in 0..bounds.x * bounds.y * bounds.z {
                        assert_eq!(
                            held[index as usize],
                            object.is_live(U32Id::from_u32(index)),
                            "{name}, object {object_id}, voxel {index}, solids {solids}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_wall_on_a_floor_splits_into_two_solids() {
        let material = U32Id::from_u32(0);

        let object = painted_object("o", TyVector3U32::new(16, 16, 16), |p| {
            (p.y < 2 || p.x < 2).then_some(material)
        });

        let masks = TileMasks::new(&object, true, false).unwrap();

        assert_eq!(
            masks.solids,
            [
                (TyVector3U32::ZERO, TyVector3U32::new(2, 16, 16)),
                (TyVector3U32::new(2, 0, 0), TyVector3U32::new(16, 2, 16)),
            ]
        );
        assert!(masks.boxes.is_empty());
    }

    #[test]
    fn a_slab_under_cells_that_cover_it_stays_in_the_march() {
        let material = U32Id::from_u32(0);

        // A floor under a checkered layer.
        let object = painted_object("o", TyVector3U32::new(16, 3, 16), |p| {
            (p.y < 2 || (p.x + p.z) % 2 == 0).then_some(material)
        });

        let masks = TileMasks::new(&object, true, false).unwrap();

        assert!(masks.solids.is_empty());
        assert_eq!(
            masks.boxes,
            [(TyVector3U32::ZERO, TyVector3U32::new(16, 3, 16))]
        );
    }
}
