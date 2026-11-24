use crate::{
    CellBox, CellGrid, Error, MaskTile, Result, TILE, TILE_CELLS, TileBox, live_boxes, tile_cell,
    to_word,
};
use ty_math::TyVector3U32;

/// The most solid boxes a tile gives up.
const SOLIDS: usize = 4;

/// The fewest cells a solid box holds.
const SOLID_CELLS: u32 = 32;

/// The most stray cells a tile gives up.
const STRAYS: usize = 16;

/// The cells of a box's face layer per stray cell it holds, at the least,
/// for the cells to come off.
const STRAY_SHARE: usize = 8;

/// An object's tiles that hold an opaque cell, as a mask march reads them.
///
/// Cell `(x, y, z)` of a tile has index `(y * 16 + x) * 16 + z`, which puts
/// a layer across y in words that follow one another. A row is the 16 cells
/// along z that share an x and a y. A tile's live cells are its opaque ones.
/// Its record cells are the live cells of each row that holds a surface
/// cell: a live cell beside a cell that is not opaque.
///
/// A tile's words:
///
/// 1. A word for each of its 256 rows. The low 16 bits hold the row's live
///    cells, and the bits above them count the tile's record cells before
///    the row
/// 2. At [`FIRST`](Self::FIRST), the record cells of the object's tiles
///    before this one
/// 3. At [`PALETTE`](Self::PALETTE), `offset | count << 16`: where the
///    tile's material ids start from its first word, and how many it holds
/// 4. From [`RECORDS`](Self::RECORDS), two words of occlusion for each
///    record cell. Face `2 * axis` looks down its axis and face
///    `2 * axis + 1` up it. A face takes 8 bits, faces 0 to 3 in the first
///    word and faces 4 and 5 in the second. A corner takes 2 of them for
///    how open it is, in thirds. Corner `u | v << 1` lies at the face's far
///    edge along the next axis where `u` is set, and along the one after
///    where `v` is
/// 5. [`SPARE`](Self::SPARE) more records, all open, where a live cell of a
///    row with no surface cell finds its place
/// 6. The tile's material ids
///
/// A face that looks into an opaque cell holds open corners. A fragment
/// that slips between two boxes' faces then shades under the sky, where
/// zeros would leave a black pixel.
#[derive(Default)]
pub struct TileMasks {
    /// Each tile that holds an opaque cell, in raster order.
    pub tiles: Vec<MaskTile>,

    /// Boxes that hold only live cells, in the tiles' order. A raster draws
    /// their faces as the cells' faces, with no march.
    pub solids: Vec<TileBox>,

    /// The box around each tile's live cells outside its solid boxes, in
    /// the tiles' order.
    pub boxes: Vec<TileBox>,

    /// The tiles' words.
    pub words: Vec<u32>,
}

impl TileMasks {
    /// The rows of a tile.
    const ROWS: usize = TILE_CELLS / TILE as usize;

    /// The spare records after a tile's records: the most live cells a row
    /// holds.
    pub const SPARE: u32 = TILE;

    /// The offset in a tile's words of the record cells before the tile.
    pub const FIRST: u32 = Self::ROWS as u32;

    /// The offset in a tile's words of the word that locates its material
    /// ids.
    pub const PALETTE: u32 = Self::FIRST + 1;

    /// The offset in a tile's words of its record cells' records.
    pub const RECORDS: u32 = Self::PALETTE + 1;

    /// The masks of `grid`'s tiles, where `opaque` says whether a material
    /// passes no light. Errors if a tile holds over 256 materials.
    ///
    /// Each tile first gives up solid boxes of two kinds, as long as it has
    /// one to give:
    ///
    /// 1. Stray cells, up to [`STRAYS`] a tile. A face of the box around
    ///    the tile's other live cells gives up its layer's cells when they
    ///    fill one part in [`STRAY_SHARE`] of the layer or less and the
    ///    layer behind it is at least half full. Each run of them along a
    ///    row makes a box
    /// 2. Slabs, up to [`SOLIDS`] a tile. A slab is the largest box of live
    ///    cells alone off a face of the box around the rest, of
    ///    [`SOLID_CELLS`] cells or more. A slab stays in the march when the
    ///    cells left would cover over half of the face it shows them
    pub fn new(grid: &CellGrid, opaque: &dyn Fn(u32) -> bool) -> Result<Self> {
        let mut masks = TileMasks::default();

        // The object's record cells so far.
        let mut record_cells = 0;

        for (low, _) in live_boxes(grid) {
            let corner = low / TILE * TILE;

            // Each cell's value: 0 when empty, else its material id plus
            // one.
            let values: Vec<u32> = (0..TILE_CELLS)
                .map(|index| grid.at((corner + tile_cell(index)).to_array().map(i64::from)))
                .collect();

            let live: Vec<bool> = values
                .iter()
                .map(|&value| value.checked_sub(1).is_some_and(opaque))
                .collect();

            if !live.contains(&true) {
                continue;
            }

            let offset = to_word(masks.words.len());
            let [low_x, low_y, low_z] = corner.to_array().map(i64::from);

            // Whether the cell `offset` from the tile's cell `index` is
            // opaque.
            let blocks = |index: usize, offset: [i64; 3]| {
                let [x, y, z] = tile_cell(index).to_array().map(i64::from);
                let cell = [
                    low_x + x + offset[0],
                    low_y + y + offset[1],
                    low_z + z + offset[2],
                ];

                grid.at(cell).checked_sub(1).is_some_and(opaque)
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
            masks.push_boxes(corner, live);

            masks.tiles.push(MaskTile {
                corner,
                offset,
                entries: (recorded.iter())
                    .map(|&index| entries[usize::from(index)])
                    .collect(),
                recorded,
            });
        }

        Ok(masks)
    }

    /// The material ids of the tile whose words start at `offset`.
    pub fn palette(&self, offset: u32) -> &[u32] {
        let palette = self.words[(offset + Self::PALETTE) as usize];
        let first = (offset + (palette & 0xffff)) as usize;

        &self.words[first..first + (palette >> 16) as usize]
    }

    /// Appends the boxes of the next tile, whose low corner is `corner` and
    /// whose cells are `live`: its solid boxes, then the box around its
    /// other live cells.
    fn push_boxes(&mut self, corner: TyVector3U32, live: Vec<bool>) {
        let place = self.tiles.len();
        let placed = |(low, high): CellBox| TileBox {
            cells: (corner + low, corner + high),
            tile: place,
        };

        let mut tile = Tile {
            left: live.clone(),
            live,
        };

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

            self.solids.extend(boxes.into_iter().map(placed));
        }

        self.boxes.extend(tile.around().map(placed));
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

        for index in (0..TILE_CELLS).filter(|&index| self.left[index]) {
            low = low.min(tile_cell(index));
            high = high.max(tile_cell(index) + 1);
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
                    && (0..TILE_CELLS)
                        .filter(|&index| {
                            let cell = tile_cell(index);

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

                for index in (0..TILE_CELLS).filter(|&index| self.left[index]) {
                    let cell = tile_cell(index);

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

#[cfg(test)]
mod tests {
    use crate::{
        CellBox, CellGrid, TileBox, TileMasks, bench_scenes, painted_object, raster_index,
        tile_cell,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxrender::RenderObject;

    /// The cells of each of `boxes`.
    fn cells(boxes: &[TileBox]) -> Vec<CellBox> {
        boxes.iter().map(|held| held.cells).collect()
    }

    /// Whether a tile's words hold the cell of index `bit` live.
    fn is_set(words: &[u32], tile: u32, bit: u32) -> bool {
        words[(tile + bit / 16) as usize] >> (bit % 16) & 1 == 1
    }

    /// The place among a tile's record cells of the live cell of index
    /// `bit`, as the shader reads it.
    fn rank(words: &[u32], tile: u32, bit: u32) -> u32 {
        let row = words[(tile + bit / 16) as usize];

        (row >> 16) + (row & ((1 << (bit % 16)) - 1)).count_ones()
    }

    #[test]
    fn a_tile_holds_its_opaque_cells_alone() {
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
        let masks = TileMasks::new(&grid, &|id| id == 0).unwrap();

        // A row runs along z, so the wall's cell and the glass cell each
        // start a row.
        assert_eq!(masks.tiles.len(), 1);
        assert_eq!(masks.tiles[0].corner, TyVector3U32::ZERO);
        assert!(is_set(&masks.words, 0, 0));
        assert!(!is_set(&masks.words, 0, 16));
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
                let masks = TileMasks::new(&grid, &|_| true).unwrap();
                let bounds = object.bounds();
                let mut held = vec![false; grid.cells.len()];

                for solid in &masks.solids {
                    let (low, high) = solid.cells;

                    for x in low.x..high.x {
                        for y in low.y..high.y {
                            for z in low.z..high.z {
                                let cell = TyVector3U32::new(x, y, z);

                                assert!(
                                    is_live(object, cell),
                                    "{name}, object {object_id}, solid cell {cell}"
                                );

                                held[raster_index(cell, bounds)] = true;
                            }
                        }
                    }
                }

                for marched in &masks.boxes {
                    let (low, high) = marched.cells;
                    let tile = &masks.tiles[marched.tile];

                    for bit in 0..4096 {
                        let set = is_set(&masks.words, tile.offset, bit);
                        let cell = tile.corner + tile_cell(bit as usize);

                        assert_eq!(
                            set,
                            is_live(object, cell),
                            "{name}, object {object_id}, cell {cell}"
                        );

                        if set && cell.cmpge(low).all() && cell.cmplt(high).all() {
                            held[raster_index(cell, bounds)] = true;
                        }
                    }
                }

                // A box lies in its tile.
                for held in masks.solids.iter().chain(&masks.boxes) {
                    assert_eq!(held.cells.0 / 16 * 16, masks.tiles[held.tile].corner);
                }

                // The record cells of the tiles so far.
                let mut before = 0;

                for tile in &masks.tiles {
                    let (corner, offset, recorded) = (tile.corner, tile.offset, &tile.recorded);

                    assert_eq!(masks.words[(offset + TileMasks::FIRST) as usize], before);

                    before += recorded.len() as u32;

                    // Every live cell is opaque here, so a surface cell is
                    // a live one beside a cell that is not.
                    let surface = |bit: u32| {
                        let cell = corner + tile_cell(bit as usize);

                        is_live(object, cell)
                            && (0..3).any(|axis| {
                                let mut up = cell;
                                let mut down = cell;

                                up[axis] += 1;
                                down[axis] = down[axis].wrapping_sub(1);

                                !is_live(object, up) || !is_live(object, down)
                            })
                    };

                    // A row's live cells are record cells where the row
                    // holds a surface cell.
                    let expected: Vec<u16> = (0..4096u32)
                        .filter(|&bit| {
                            let row = bit / 16 * 16;

                            is_set(&masks.words, offset, bit) && (row..row + 16).any(surface)
                        })
                        .map(|bit| bit as u16)
                        .collect();

                    assert_eq!(*recorded, expected, "{name}, object {object_id}");

                    for (place, (&bit, &entry)) in recorded.iter().zip(&tile.entries).enumerate() {
                        let cell = corner + tile_cell(usize::from(bit));

                        assert_eq!(rank(&masks.words, offset, u32::from(bit)) as usize, place);
                        assert_eq!(
                            Some(U32Id::from_u32(masks.palette(offset)[usize::from(entry)])),
                            object.voxel_material(object.voxel_id(cell).unwrap()),
                            "{name}, object {object_id}, cell {cell}"
                        );
                    }

                    // A live cell of any other row has a place within the
                    // spare records.
                    for bit in (0..4096).filter(|&bit| is_set(&masks.words, offset, bit)) {
                        assert!(
                            rank(&masks.words, offset, bit)
                                < recorded.len() as u32 + TileMasks::SPARE
                        );
                    }
                }

                for (index, &held) in (0..).zip(&held) {
                    assert_eq!(
                        held,
                        object.is_live(U32Id::from_u32(index)),
                        "{name}, object {object_id}, voxel {index}"
                    );
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

        let grid = CellGrid::new(&object, &|id| id);
        let masks = TileMasks::new(&grid, &|_| true).unwrap();

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

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), &|_| true).unwrap();

        assert_eq!(
            cells(&masks.solids),
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

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), &|_| true).unwrap();

        assert_eq!(
            cells(&masks.solids),
            [
                (TyVector3U32::new(3, 3, 5), TyVector3U32::new(4, 4, 7)),
                (TyVector3U32::new(10, 3, 10), TyVector3U32::new(11, 4, 11)),
            ]
        );
        assert_eq!(
            cells(&masks.boxes),
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

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), &|_| true).unwrap();

        assert!(masks.solids.is_empty());
        assert_eq!(
            cells(&masks.boxes),
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

        let masks = TileMasks::new(&CellGrid::new(&object, &|id| id), &|_| true).unwrap();

        assert!(masks.solids.is_empty());
        assert_eq!(
            cells(&masks.boxes),
            [(TyVector3U32::ZERO, TyVector3U32::new(16, 3, 16))]
        );
    }
}
