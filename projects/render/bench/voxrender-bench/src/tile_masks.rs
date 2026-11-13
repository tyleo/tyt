use crate::{CellGrid, Error, Result, live_boxes};
use ty_math::TyVector3U32;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The cells of a tile.
const CELLS: usize = (TILE * TILE * TILE) as usize;

/// The most solid boxes a tile gives up.
const SOLIDS: usize = 4;

/// The fewest cells a solid box holds.
const SOLID_CELLS: u32 = 32;

/// The most stray cells a tile gives up.
const STRAYS: usize = 16;

/// The cells of a box's face layer per stray cell it holds, at the least,
/// for the cells to come off.
const STRAY_SHARE: usize = 8;

/// A box of cells: its low cell and the corner past its high cell.
type CellBox = (TyVector3U32, TyVector3U32);

/// An object's non-empty tiles as a mask march reads them.
///
/// Cell `(x, y, z)` of a tile has index `(y * 16 + x) * 16 + z`. A layer of
/// cells across y then sits in words that follow one another, which a
/// floor's far tiles read a few pixels each. S5's view ran 5% faster on
/// Quest than with x first.
///
/// A tile's words hold a bit per cell, 32 cells a word: a cell is live
/// where bit `index % 32` of word `index / 32` is set.
///
/// A shaded view's tile holds other words. Its live cells are its opaque
/// cells alone. Its record cells are the live cells of each row that holds
/// a surface cell. A row is the 16 cells along z that share an x and a y. A
/// surface cell is a live cell beside a cell that is not opaque. The words:
///
/// 1. A word for each row, 256 in the cells' index order. Its low 16 bits
///    hold the row's cells, and the bits above them the tile's record cells
///    before the row. The load that tells a march a cell is live then gives
///    the cell's place among the record cells: S5's view ran 3% faster on
///    Quest than with two more loads for that place
/// 2. At [`FIRST`](Self::FIRST), the record cells of the object's tiles
///    before this one
/// 3. At [`PALETTE`](Self::PALETTE), one word of `offset | count << 16`:
///    where the tile's material ids start from its first word, and how many
///    it holds
/// 4. From [`RECORDS`](Self::RECORDS), two words for each record cell, in
///    the cells' index order. Face `2 * axis` looks down its axis and face
///    `2 * axis + 1` up it. A face takes 8 bits of occlusion: faces 0 to 3
///    fill the first word from its low bits, and faces 4 and 5 the second.
///    A corner takes 2 of them for how open it is, in thirds. Corner
///    `u | v << 1` lies at the face's far edge along the axis after the
///    face's where `u` is set, and along the axis after that where `v` is.
/// 5. Sixteen more records. A fragment can reach a live cell of a row with
///    no surface cell, and the place it works out for the cell ends within
///    these
/// 6. The tile's material ids
///
/// A face that looks into an opaque cell holds open corners, as a spare
/// record's faces do. A fragment reaches one where it slips between two
/// boxes' faces, or before a nearer box covers it. It then shades its
/// material under the sky, where zeros would leave a black pixel.
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

    /// Each non-empty tile's low corner, in the order of the tiles' words.
    pub corners: Vec<TyVector3U32>,

    /// The offset in [`words`](Self::words) of each non-empty tile, in the
    /// same order.
    pub offsets: Vec<u32>,

    /// The indices of each non-empty tile's record cells in index order,
    /// in the order of the tiles' words. Empty unless the view is shaded.
    pub recorded: Vec<Vec<u16>>,

    /// Each record cell's material, as its place among its tile's material
    /// ids, in the order of [`recorded`](Self::recorded).
    pub entries: Vec<Vec<u8>>,

    /// Each non-empty tile's words.
    pub words: Vec<u32>,
}

impl TileMasks {
    /// The words of one tile's bits.
    const BIT_WORDS: usize = CELLS / 32;

    /// The rows of a tile.
    const ROWS: usize = CELLS / TILE as usize;

    /// The spare records after a shaded tile's records: the most live
    /// cells a row holds.
    pub const SPARE: u32 = TILE;

    /// The offset in a shaded tile's words of the record cells before the
    /// tile.
    pub const FIRST: u32 = Self::ROWS as u32;

    /// The offset in a shaded tile's words of the word that locates its
    /// material ids.
    pub const PALETTE: u32 = Self::FIRST + 1;

    /// The offset in a shaded tile's words of its record cells' records.
    pub const RECORDS: u32 = Self::PALETTE + 1;

    /// The masks of `grid`'s non-empty tiles. With `opaque`, which says
    /// whether a material passes no light, each tile's words go on for a
    /// shaded view. Errors if a tile holds over 256 materials.
    ///
    /// When `solids`, each tile first gives up solid boxes of two kinds, as
    /// long as it has one to give:
    ///
    /// 1. Stray cells, up to 16 a tile. A face of the box around the tile's
    ///    other live cells gives up its layer's cells when they fill an
    ///    eighth of the layer or less and the layer behind it is at least
    ///    half full. Each run of them along a row makes a box, and the box
    ///    around the rest shrinks by the layer
    /// 2. Slabs, up to four a tile. A slab is the largest box of live cells
    ///    alone off a face of the box around the rest. It holds 32 cells or
    ///    more. A slab stays in the march when the cells left would cover over
    ///    half of the face it shows them
    pub fn new(
        grid: &CellGrid,
        solids: bool,
        opaque: Option<&dyn Fn(u32) -> bool>,
    ) -> Result<Self> {
        let bounds = grid.bounds;
        let cells = &grid.cells;

        // The value of the cell at `cell`: 0 when empty or outside the
        // object, else its material id plus one.
        let value = |cell: [i64; 3]| {
            let inside = (0..3).all(|axis| cell[axis] >= 0 && cell[axis] < i64::from(bounds[axis]));

            if inside {
                let [x, y, z] = cell.map(|coordinate| coordinate as usize);

                cells[(x * bounds.y as usize + y) * bounds.z as usize + z]
            } else {
                0
            }
        };

        let mut masks = TileMasks {
            solids: Vec::new(),
            solid_tiles: Vec::new(),
            boxes: Vec::new(),
            tiles: Vec::new(),
            corners: Vec::new(),
            offsets: Vec::new(),
            recorded: Vec::new(),
            entries: Vec::new(),
            words: Vec::new(),
        };

        // The object's record cells so far.
        let mut record_cells = 0;

        for (low, _) in live_boxes(grid, TILE) {
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

            // A shaded view's tile holds its opaque cells alone.
            let live: Vec<bool> = values
                .iter()
                .map(|&value| {
                    value
                        .checked_sub(1)
                        .is_some_and(|id| opaque.is_none_or(|opaque| opaque(id)))
                })
                .collect();

            if !live.contains(&true) {
                continue;
            }

            let offset =
                u32::try_from(masks.words.len()).expect("an object's mask words number under 2^32");

            let Some(opaque) = opaque else {
                masks.words.resize(masks.words.len() + Self::BIT_WORDS, 0);

                for index in (0..CELLS).filter(|&index| live[index]) {
                    masks.words[offset as usize + index / 32] |= 1 << (index % 32);
                }

                masks.push_boxes(corner, offset, live, solids);

                continue;
            };

            let [low_x, low_y, low_z] = corner.to_array().map(i64::from);

            // Whether the cell `offset` from the tile's cell `index` is
            // opaque.
            let blocks = |index: usize, offset: [i64; 3]| {
                let [x, y, z] = position(index).to_array().map(i64::from);
                let cell = [
                    low_x + x + offset[0],
                    low_y + y + offset[1],
                    low_z + z + offset[2],
                ];

                value(cell).checked_sub(1).is_some_and(opaque)
            };

            let surface =
                |index: usize| live[index] && (0..6).any(|face| !blocks(index, toward(face, 0, 0)));

            // Each row's word, and the record cells its rows give the tile.
            let mut recorded: Vec<u16> = Vec::new();

            for row in 0..Self::ROWS {
                let cells = row * TILE as usize..(row + 1) * TILE as usize;
                let before = recorded.len() as u32;
                let mut bits = 0;

                for index in cells.clone().filter(|&index| live[index]) {
                    bits |= 1 << (index % TILE as usize);
                }

                if cells.clone().any(surface) {
                    recorded.extend(cells.filter(|&index| live[index]).map(|index| index as u16));
                }

                masks.words.push(bits | before << 16);
            }

            let (entries, palette) = palette_of(&values)?;
            let count = recorded.len() as u32;
            let ids = Self::RECORDS + 2 * (count + Self::SPARE);

            masks.words.extend([
                record_cells,
                ids | u32::try_from(palette.len()).expect("a tile holds 256 materials") << 16,
            ]);

            record_cells += count;

            for &index in &recorded {
                let index = usize::from(index);
                let mut words = [0; 2];

                for face in 0..6 {
                    if blocks(index, toward(face, 0, 0)) {
                        words[face / 4] |= 0xff << (face % 4 * 8);

                        continue;
                    }

                    for corner in 0..4 {
                        let u = if corner & 1 == 1 { 1 } else { -1 };
                        let v = if corner & 2 == 2 { 1 } else { -1 };
                        let first = blocks(index, toward(face, u, 0));
                        let second = blocks(index, toward(face, 0, v));
                        let across = blocks(index, toward(face, u, v));

                        // Both edge cells together close the corner.
                        let open = if first && second {
                            0
                        } else {
                            3 - u32::from(first) - u32::from(second) - u32::from(across)
                        };

                        words[face / 4] |= open << (face % 4 * 8 + corner * 2);
                    }
                }

                masks.words.extend(words);
            }

            for _ in 0..Self::SPARE {
                masks.words.extend([u32::MAX, 0xffff]);
            }

            masks.words.extend(palette);
            masks.entries.push(
                recorded
                    .iter()
                    .map(|&index| entries[usize::from(index)])
                    .collect(),
            );
            masks.recorded.push(recorded);
            masks.push_boxes(corner, offset, live, solids);
        }

        Ok(masks)
    }

    /// Appends the boxes of the tile at `corner`, whose words start at
    /// `offset` and whose cells are `live`: its solid boxes when `solids`,
    /// then the box around its other live cells.
    fn push_boxes(&mut self, corner: TyVector3U32, offset: u32, live: Vec<bool>, solids: bool) {
        self.corners.push(corner);
        self.offsets.push(offset);

        let mut tile = Tile {
            left: live.clone(),
            live,
        };

        if solids {
            let mut strays = 0;
            let mut slabs = 0;

            while let Some(around) = tile.around() {
                let boxes = if let Some((cells, boxes)) = tile.strays(around, STRAYS - strays) {
                    strays += cells;

                    boxes
                } else if let Some(slab) = (slabs < SOLIDS).then(|| tile.slab(around)).flatten() {
                    slabs += 1;

                    vec![slab]
                } else {
                    break;
                };

                for (low, high) in boxes {
                    self.solids.push((corner + low, corner + high));
                    self.solid_tiles.push(offset);
                }
            }
        }

        if let Some((low, high)) = tile.around() {
            self.boxes.push((corner + low, corner + high));
            self.tiles.push(offset);
        }
    }
}

/// A tile's material ids in the order its cells first hold them, from each
/// cell's `values`: 0 when empty, else its material id plus one. Returns
/// each cell's place among the ids, 0 for an empty cell, and then the ids.
/// Errors if the tile holds over 256 materials.
fn palette_of(values: &[u32]) -> Result<(Vec<u8>, Vec<u32>)> {
    let mut palette: Vec<u32> = Vec::new();

    // A place past 255 wraps here, and the count below then errors.
    let entries = values
        .iter()
        .map(|&value| match value.checked_sub(1) {
            Some(id) => palette
                .iter()
                .position(|&held| held == id)
                .unwrap_or_else(|| {
                    palette.push(id);

                    palette.len() - 1
                }) as u8,

            None => 0,
        })
        .collect();

    if palette.len() > 256 {
        return Err(Error::TileMaterials {
            materials: palette.len(),
        });
    }

    Ok((entries, palette))
}

/// The step from a cell to the cell its face `face` looks into, moved `u`
/// along the axis after the face's and `v` along the axis after that.
fn toward(face: usize, u: i64, v: i64) -> [i64; 3] {
    let axis = face / 2;
    let mut step = [0; 3];

    step[axis] = if face % 2 == 1 { 1 } else { -1 };
    step[(axis + 1) % 3] = u;
    step[(axis + 2) % 3] = v;

    step
}

/// One tile's cells while its solid boxes come off. A cell's index is
/// `(y * 16 + x) * 16 + z`.
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

    /// Takes the cells left in the emptiest layer on a face of `around`,
    /// the box around the cells left. Returns their count and a box for
    /// each run of them along a row. Takes nothing and returns `None`
    /// unless a layer holds at most `room` cells that fill at most one part
    /// in [`STRAY_SHARE`] of it, before a layer that is at least half full.
    fn strays(&mut self, (low, high): CellBox, room: usize) -> Option<(usize, Vec<CellBox>)> {
        let extent = high - low;
        let mut best: Option<(usize, Vec<CellBox>)> = None;

        for axis in 0..3 {
            let across = (extent.x * extent.y * extent.z / extent[axis]) as usize;

            // Each face's layer, with the layer behind it.
            for (at, behind) in [
                (low[axis], low[axis] + 1),
                (high[axis] - 1, high[axis].wrapping_sub(2)),
            ] {
                let mut layer_low = low;
                let mut layer_high = high;

                layer_low[axis] = at;
                layer_high[axis] = at + 1;

                // Rays that pass the strays stop soon only where the layer
                // behind them is at least half full.
                let mut behind_low = low;
                let mut behind_high = high;

                behind_low[axis] = behind;
                behind_high[axis] = behind.wrapping_add(1);

                let backed = extent[axis] > 1
                    && (0..CELLS)
                        .filter(|&index| {
                            let cell = position(index);

                            self.left[index]
                                && cell.cmpge(behind_low).all()
                                && cell.cmplt(behind_high).all()
                        })
                        .count()
                        * 2
                        >= across;

                if !backed {
                    continue;
                }

                // Index order runs along z, or along x in a layer across z,
                // so a row's cells follow one another.
                let row = if axis == 2 { 0 } else { 2 };
                let mut cells = 0;
                let mut runs: Vec<CellBox> = Vec::new();

                for index in (0..CELLS).filter(|&index| self.left[index]) {
                    let cell = position(index);

                    if cell.cmplt(layer_low).any() || cell.cmpge(layer_high).any() {
                        continue;
                    }

                    cells += 1;

                    // Where a run that ends just before the cell ends.
                    let mut end = cell + 1;

                    end[row] = cell[row];

                    match runs.last_mut() {
                        Some((_, run_high)) if *run_high == end => run_high[row] += 1,
                        _ => runs.push((cell, cell + 1)),
                    }
                }

                let fewer = best.as_ref().is_none_or(|&(least, _)| cells < least);

                if cells * STRAY_SHARE <= across && cells <= room && fewer {
                    best = Some((cells, runs));
                }
            }
        }

        let (cells, runs) = best?;

        for &(run_low, run_high) in &runs {
            self.remove(run_low, run_high);
        }

        Some((cells, runs))
    }

    /// Whether every cell from `low` to before `high` is live.
    fn all_live(&self, low: TyVector3U32, high: TyVector3U32) -> bool {
        (low.x..high.x).all(|x| {
            (low.y..high.y)
                .all(|y| (low.z..high.z).all(|z| self.live[((y * TILE + x) * TILE + z) as usize]))
        })
    }

    /// Takes the cells from `low` to before `high` out of the cells left.
    fn remove(&mut self, low: TyVector3U32, high: TyVector3U32) {
        for x in low.x..high.x {
            for y in low.y..high.y {
                for z in low.z..high.z {
                    self.left[((y * TILE + x) * TILE + z) as usize] = false;
                }
            }
        }
    }
}

/// The cell of `index` in a tile.
fn position(index: usize) -> TyVector3U32 {
    let index = index as u32;

    TyVector3U32::new(index / TILE % TILE, index / (TILE * TILE), index % TILE)
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, TileMasks, bench_scenes, painted_object};
    use branded_id::U32Id;
    use std::collections::HashMap;
    use ty_math::TyVector3U32;
    use voxrender::RenderObject;

    /// Whether a tile's words hold the cell of index `bit` live. `shaded`
    /// says how the tile's words lie.
    fn is_set(words: &[u32], tile: u32, bit: u32, shaded: bool) -> bool {
        let (cells, word) = if shaded { (16, 16) } else { (32, 32) };

        words[(tile + bit / word) as usize] >> (bit % cells) & 1 == 1
    }

    /// The place among a shaded tile's record cells of the live cell of
    /// index `bit`, as the shader reads it.
    fn rank(words: &[u32], tile: u32, bit: u32) -> u32 {
        let row = words[(tile + bit / 16) as usize];

        (row >> 16) + (row & ((1 << (bit % 16)) - 1)).count_ones()
    }

    /// The material id a shaded tile's words hold for the place `entry`
    /// among its material ids.
    fn material(words: &[u32], tile: u32, entry: u8) -> u32 {
        let palette = words[(tile + TileMasks::PALETTE) as usize];

        assert!(u32::from(entry) < palette >> 16);

        words[(tile + (palette & 0xffff) + u32::from(entry)) as usize]
    }

    #[test]
    fn a_shaded_tile_holds_its_opaque_cells_alone() {
        // A wall cell beside a glass cell, and a tile of glass alone.
        let object = painted_object(
            "pane",
            TyVector3U32::new(18, 1, 1),
            |position| match position.x {
                0 => Some(U32Id::from_u32(0)),
                1 | 17 => Some(U32Id::from_u32(1)),
                _ => None,
            },
        );

        let grid = CellGrid::new(&object, &|id| id);
        let masks = TileMasks::new(&grid, false, Some(&|id| id == 0)).unwrap();

        // A row runs along z, so the wall's cell and the glass cell each
        // start a row.
        assert_eq!(masks.corners, [TyVector3U32::ZERO]);
        assert!(is_set(&masks.words, 0, 0, true));
        assert!(!is_set(&masks.words, 0, 16, true));

        // With no word on what is opaque, every cell is live.
        assert_eq!(TileMasks::new(&grid, false, None).unwrap().boxes.len(), 2);
    }

    /// Whether `cell` of `object` is live.
    fn is_live(object: &RenderObject, cell: TyVector3U32) -> bool {
        object
            .voxel_id(cell)
            .is_some_and(|voxel_id| object.is_live(voxel_id))
    }

    #[test]
    fn the_solids_and_masks_hold_every_live_cell_of_each_scene() {
        for (name, scene) in bench_scenes() {
            for (object_id, object) in scene.iter_objects() {
                let grid = CellGrid::new(object, &|id| id);

                for (solids, materials) in [(false, false), (true, false), (true, true)] {
                    let every = |_| true;
                    let opaque: Option<&dyn Fn(u32) -> bool> = materials.then_some(&every);
                    let masks = TileMasks::new(&grid, solids, opaque).unwrap();
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
                            let at = TyVector3U32::new(bit / 16 % 16, bit / 256, bit % 16);
                            let set = is_set(&masks.words, tile, bit, materials);
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

                        // Each tile's place among the tiles, and the record
                        // cells before it.
                        let mut before = 0;

                        let ordinals: HashMap<_, _> = (0..)
                            .zip(masks.corners.iter().zip(&masks.recorded))
                            .map(|(ordinal, (&corner, recorded))| {
                                let entry = (corner.to_array(), (ordinal, before));

                                before += recorded.len() as u32;

                                entry
                            })
                            .collect();

                        for (&tile, &(low, _)) in tiles.chain(solid_tiles) {
                            let corner = low / 16 * 16;
                            let (ordinal, before): (usize, u32) = ordinals[&corner.to_array()];
                            let recorded = &masks.recorded[ordinal];

                            assert_eq!(masks.words[(tile + TileMasks::FIRST) as usize], before);

                            // Every live cell is opaque here, so a surface
                            // cell is a live one beside a cell that is not.
                            let surface = |bit: u32| {
                                let at = TyVector3U32::new(bit / 16 % 16, bit / 256, bit % 16);
                                let cell = corner + at;

                                is_live(object, cell)
                                    && (0..3).any(|axis| {
                                        let mut up = cell;
                                        let mut down = cell;

                                        up[axis] += 1;
                                        down[axis] = down[axis].wrapping_sub(1);

                                        !is_live(object, up) || !is_live(object, down)
                                    })
                            };

                            // A row's live cells are record cells where the
                            // row holds a surface cell.
                            let expected: Vec<u16> = (0..4096u32)
                                .filter(|&bit| {
                                    let row = bit / 16 * 16;

                                    is_set(&masks.words, tile, bit, true)
                                        && (row..row + 16).any(surface)
                                })
                                .map(|bit| bit as u16)
                                .collect();

                            assert_eq!(*recorded, expected, "{name}, object {object_id}");

                            for (place, &bit) in recorded.iter().enumerate() {
                                let bit = u32::from(bit);
                                let rank = rank(&masks.words, tile, bit);
                                let at = TyVector3U32::new(bit / 16 % 16, bit / 256, bit % 16);
                                let cell = corner + at;

                                assert_eq!(rank as usize, place);

                                let entry = masks.entries[ordinal][place];

                                assert_eq!(
                                    Some(U32Id::from_u32(material(&masks.words, tile, entry))),
                                    object.voxel_material(object.voxel_id(cell).unwrap()),
                                    "{name}, object {object_id}, cell {cell}"
                                );
                            }

                            // A live cell of any other row has a place
                            // within the spare records.
                            for bit in
                                (0..4096).filter(|&bit| is_set(&masks.words, tile, bit, true))
                            {
                                assert!(
                                    rank(&masks.words, tile, bit)
                                        < recorded.len() as u32 + TileMasks::SPARE
                                );
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
    fn a_floor_cell_beside_a_wall_holds_its_corners_occlusion() {
        let material = U32Id::from_u32(0);

        let object = painted_object("o", TyVector3U32::new(16, 16, 16), |p| {
            (p.y < 1 || p.x < 1).then_some(material)
        });

        let every = |_| true;
        let grid = CellGrid::new(&object, &|id| id);
        let masks = TileMasks::new(&grid, false, Some(&every)).unwrap();

        // The floor's cell at x = 1 and z = 1, beside the wall.
        let rank = rank(&masks.words, 0, 16 + 1);
        let words = (TileMasks::RECORDS + rank * 2) as usize;

        // Its face up y has its corners along z and then x. The wall
        // closes the two at low x by two thirds. Its face down y looks out
        // of the object. Its other faces look into opaque cells, which
        // leaves them open.
        assert_eq!(masks.words[words], 0b1111_0101 << 24 | 0xff_ffff);
        assert_eq!(masks.words[words + 1], 0xffff);
    }

    #[test]
    fn a_wall_on_a_floor_splits_into_two_solids() {
        let material = U32Id::from_u32(0);

        let object = painted_object("o", TyVector3U32::new(16, 16, 16), |p| {
            (p.y < 2 || p.x < 2).then_some(material)
        });

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), true, None).unwrap();

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
    fn stray_cells_over_a_floor_come_off_as_boxes() {
        let material = U32Id::from_u32(0);

        // A floor under a checkered layer, with three cells above it.
        let object = painted_object("o", TyVector3U32::new(16, 4, 16), |p| {
            let stray = p.y == 3 && [(3, 5), (3, 6), (10, 10)].contains(&(p.x, p.z));

            (p.y < 2 || (p.y == 2 && (p.x + p.z) % 2 == 0) || stray).then_some(material)
        });

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), true, None).unwrap();

        assert_eq!(
            masks.solids,
            [
                (TyVector3U32::new(3, 3, 5), TyVector3U32::new(4, 4, 7)),
                (TyVector3U32::new(10, 3, 10), TyVector3U32::new(11, 4, 11)),
            ]
        );
        assert_eq!(
            masks.boxes,
            [(TyVector3U32::ZERO, TyVector3U32::new(16, 3, 16))]
        );
    }

    #[test]
    fn strays_over_a_sparse_layer_stay_in_the_march() {
        let material = U32Id::from_u32(0);

        // One cell over a layer that holds two cells.
        let object = painted_object("o", TyVector3U32::new(16, 2, 16), |p| {
            [(0, 0, 0), (15, 0, 15), (5, 1, 5)]
                .contains(&(p.x, p.y, p.z))
                .then_some(material)
        });

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), true, None).unwrap();

        assert!(masks.solids.is_empty());
        assert_eq!(
            masks.boxes,
            [(TyVector3U32::ZERO, TyVector3U32::new(16, 2, 16))]
        );
    }

    #[test]
    fn a_slab_under_cells_that_cover_it_stays_in_the_march() {
        let material = U32Id::from_u32(0);

        // A floor under a checkered layer.
        let object = painted_object("o", TyVector3U32::new(16, 3, 16), |p| {
            (p.y < 2 || (p.x + p.z) % 2 == 0).then_some(material)
        });

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), true, None).unwrap();

        assert!(masks.solids.is_empty());
        assert_eq!(
            masks.boxes,
            [(TyVector3U32::ZERO, TyVector3U32::new(16, 3, 16))]
        );
    }
}
