use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxrender::{BRenderMaterial, RenderObject};

/// The cells along each axis of a root tile.
const TILE: u32 = 16;

/// The children along each axis of a node, and the cells along each axis of
/// a leaf.
const BRANCH: u32 = 4;

/// The shift that puts a slot's tag in its top 3 bits.
const TAG_SHIFT: u32 = 29;

/// The bits of a slot below its tag.
const PAYLOAD: u32 = (1 << TAG_SHIFT) - 1;

/// The tag of a slot holding one value over its region.
const UNIFORM: u32 = 0;

/// The tag of a slot pointing to a node.
const NODE: u32 = 1;

/// The tag of a slot pointing to a leaf of 1-bit cells. Each tag above it
/// doubles the bits.
const LEAF: u32 = 2;

/// The tag of an empty tile's slot holding the box of empty tiles around it.
const EMPTY: u32 = 6;

/// The most tiles a box reaches past its tile on each side.
const REACH: u32 = 15;

/// An object's cells in the 64-tree of research section 3.4, flattened into
/// words for the GPU. A cell's value is 0 when empty, else its material id
/// plus one.
///
/// The words start with a dense grid of 16^3 tiles in raster order, one slot
/// word each. A slot's top 3 bits tag it, and its low 29 bits hold its
/// payload:
///
/// 1. Tag 0 holds one value over the slot's region
/// 2. Tag 1 points to a node over the region's 4^3 children: two mask words
///    with a bit set for each non-empty child, then one slot per set bit in
///    bit order
/// 3. Tags 2 to 5 point to a leaf of 4^3 cells at 1, 2, 4, or 8 bits each.
///    The packed cell indices come first, then the palette of values they
///    index. Cell `i`'s index sits at bit `i * bits % 32` of the leaf's word
///    `i * bits / 32`
/// 4. Tag 6 marks an empty tile. Its payload holds how many empty tiles the
///    box around it reaches past the tile, 4 bits a side, in the order -x,
///    -y, -z, +x, +y, +z from the low bits
///
/// A child's bit, like a cell's, is `x * 16 + y * 4 + z` over its 4^3
/// positions. Bits 0 to 31 sit in a node's first mask word. Cells past the
/// object's bounds are empty.
#[derive(Clone, Debug, PartialEq)]
pub struct Tree64 {
    bounds: TyVector3U32,

    words: Vec<u32>,
}

impl Tree64 {
    /// The tree of `object`'s cells. Errors if a material id or a word
    /// offset is past a slot's payload.
    pub fn new(object: &RenderObject) -> Result<Self> {
        let bounds = object.bounds();
        let tiles = tile_counts(bounds);
        let mut words = vec![0; (tiles.x * tiles.y * tiles.z) as usize];

        for x in 0..tiles.x {
            for y in 0..tiles.y {
                for z in 0..tiles.z {
                    let tile = TyVector3U32::new(x, y, z);

                    // Each child's 64 cells run together, in bit order.
                    let mut cells = [0; 4096];

                    for (index, cell) in (0..).zip(&mut cells) {
                        let position = tile * TILE
                            + branch_position(index / 64) * BRANCH
                            + branch_position(index % 64);

                        *cell = value(object, position)?;
                    }

                    words[raster(tiles, tile)] = if is_uniform(&cells) {
                        slot(UNIFORM, cells[0])
                    } else {
                        push_node(&mut words, &cells)?
                    };
                }
            }
        }

        let filled = FilledTiles::new(tiles, &words);

        for x in 0..tiles.x {
            for y in 0..tiles.y {
                for z in 0..tiles.z {
                    let tile = TyVector3U32::new(x, y, z);
                    let index = raster(tiles, tile);

                    if words[index] == slot(UNIFORM, 0) {
                        let reach = filled.empty_reach(tile);

                        if reach != 0 {
                            words[index] = slot(EMPTY, reach);
                        }
                    }
                }
            }
        }

        Ok(Tree64 { bounds, words })
    }

    /// The object's grid size in voxels.
    pub fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    /// The flattened tree.
    pub fn words(&self) -> &[u32] {
        &self.words
    }

    /// The material the cell at `position` holds, or `None` where the cell
    /// is empty or outside the grid. Reads the words as the GPU walk does.
    pub fn material(&self, position: TyVector3U32) -> Option<U32Id<BRenderMaterial>> {
        if !position.cmplt(self.bounds).all() {
            return None;
        }

        let tiles = tile_counts(self.bounds);
        let mut slot = self.words[raster(tiles, position / TILE)];
        let mut local = position % TILE;
        let mut size = TILE;

        let value = loop {
            let offset = (slot & PAYLOAD) as usize;

            match slot >> TAG_SHIFT {
                UNIFORM => break slot & PAYLOAD,

                EMPTY => break 0,

                NODE => {
                    size /= BRANCH;

                    let bit = branch_bit(local / size);
                    let mask =
                        u64::from(self.words[offset]) | u64::from(self.words[offset + 1]) << 32;

                    if mask >> bit & 1 == 0 {
                        break 0;
                    }

                    local %= size;
                    slot = self.words[offset + 2 + (mask & ((1 << bit) - 1)).count_ones() as usize];
                }

                tag => {
                    let bits = 1 << (tag - LEAF);
                    let start = branch_bit(local) * bits;
                    let index = self.words[offset + (start / 32) as usize] >> (start % 32)
                        & ((1 << bits) - 1);

                    break self.words[offset + 2 * bits as usize + index as usize];
                }
            }
        };

        value.checked_sub(1).map(U32Id::from_u32)
    }
}

/// The tiles along each axis over `bounds`.
fn tile_counts(bounds: TyVector3U32) -> TyVector3U32 {
    (bounds + TILE - 1) / TILE
}

/// The index of `position` in a raster grid of `size`.
fn raster(size: TyVector3U32, position: TyVector3U32) -> usize {
    ((position.x * size.y + position.y) * size.z + position.z) as usize
}

/// The position of `bit` among 4^3.
fn branch_position(bit: u32) -> TyVector3U32 {
    TyVector3U32::new(bit / 16, bit / 4 % 4, bit % 4)
}

/// The bit of `position` among 4^3.
fn branch_bit(position: TyVector3U32) -> u32 {
    position.x * 16 + position.y * 4 + position.z
}

/// The value of `object`'s cell at `position`.
fn value(object: &RenderObject, position: TyVector3U32) -> Result<u32> {
    let Some(voxel_id) = object.voxel_id(position) else {
        return Ok(0);
    };

    let Some(material_id) = object.voxel_material(voxel_id) else {
        return Ok(0);
    };

    match material_id.to_u32().checked_add(1) {
        Some(value) if value <= PAYLOAD => Ok(value),
        _ => Err(Error::TreeMaterial { material_id }),
    }
}

/// Whether every one of `values` is the first.
fn is_uniform(values: &[u32]) -> bool {
    values.iter().all(|&value| value == values[0])
}

/// The slot of `tag` over `payload`.
fn slot(tag: u32, payload: u32) -> u32 {
    tag << TAG_SHIFT | payload
}

/// The slot of a word offset, or an error past a slot's payload.
fn offset_slot(tag: u32, offset: usize) -> Result<u32> {
    match u32::try_from(offset) {
        Ok(offset) if offset <= PAYLOAD => Ok(slot(tag, offset)),
        _ => Err(Error::TreeSize { offset }),
    }
}

/// The count of filled tiles in each box from the grid's low corner, for
/// counting a box's filled tiles in 8 reads.
struct FilledTiles {
    tiles: TyVector3U32,

    counts: Vec<u32>,
}

impl FilledTiles {
    /// The counts over the grid of `tiles` whose slots open `words`.
    fn new(tiles: TyVector3U32, words: &[u32]) -> Self {
        let size = tiles + 1;
        let mut counts = vec![0; (size.x * size.y * size.z) as usize];

        for x in 1..size.x {
            for y in 1..size.y {
                for z in 1..size.z {
                    let at = |dx: u32, dy: u32, dz: u32| {
                        counts[raster(size, TyVector3U32::new(x - dx, y - dy, z - dz))]
                    };

                    let tile = TyVector3U32::new(x - 1, y - 1, z - 1);
                    let filled = u32::from(words[raster(tiles, tile)] != slot(UNIFORM, 0));

                    // Inclusion and exclusion over the 7 smaller boxes.
                    let count = filled + at(1, 0, 0) + at(0, 1, 0) + at(0, 0, 1) + at(1, 1, 1)
                        - at(1, 1, 0)
                        - at(1, 0, 1)
                        - at(0, 1, 1);

                    counts[raster(size, TyVector3U32::new(x, y, z))] = count;
                }
            }
        }

        FilledTiles { tiles, counts }
    }

    /// Whether every tile from `low` to before `high` is empty.
    fn is_empty(&self, low: TyVector3U32, high: TyVector3U32) -> bool {
        let size = self.tiles + 1;
        let at = |x: u32, y: u32, z: u32| self.counts[raster(size, TyVector3U32::new(x, y, z))];

        at(high.x, high.y, high.z)
            + at(low.x, low.y, high.z)
            + at(low.x, high.y, low.z)
            + at(high.x, low.y, low.z)
            == at(low.x, low.y, low.z)
                + at(high.x, high.y, low.z)
                + at(high.x, low.y, high.z)
                + at(low.x, high.y, high.z)
    }

    /// The packed reach of the box grown around the empty `tile`. The box
    /// grows a layer at a time on each side in turn. A side stops at the
    /// grid's edge, at a filled tile, or at `REACH` tiles.
    fn empty_reach(&self, tile: TyVector3U32) -> u32 {
        let mut low = tile;
        let mut high = tile + 1;
        let mut grew = true;

        while grew {
            grew = false;

            for axis in 0..3 {
                if tile[axis] - low[axis] < REACH && low[axis] > 0 {
                    let mut layer = high;
                    layer[axis] = low[axis];

                    let mut start = low;
                    start[axis] -= 1;

                    if self.is_empty(start, layer) {
                        low[axis] -= 1;
                        grew = true;
                    }
                }

                if high[axis] - 1 - tile[axis] < REACH && high[axis] < self.tiles[axis] {
                    let mut layer = low;
                    layer[axis] = high[axis];

                    let mut end = high;
                    end[axis] += 1;

                    if self.is_empty(layer, end) {
                        high[axis] += 1;
                        grew = true;
                    }
                }
            }
        }

        let below = tile - low;
        let above = high - 1 - tile;

        below.x | below.y << 4 | below.z << 8 | above.x << 12 | above.y << 16 | above.z << 20
    }
}

/// Appends the node over a tile's `cells` and its leaves to `words`, and
/// returns its slot.
fn push_node(words: &mut Vec<u32>, cells: &[u32]) -> Result<u32> {
    let node = words.len();

    let live: Vec<_> = (0..)
        .zip(cells.chunks(64))
        .filter(|(_, child)| child.iter().any(|&value| value != 0))
        .collect();

    let mask = live.iter().fold(0u64, |mask, (bit, _)| mask | 1 << bit);

    words.extend([mask as u32, (mask >> 32) as u32]);

    let first = words.len();

    words.resize(first + live.len(), 0);

    for (rank, (_, child)) in live.into_iter().enumerate() {
        words[first + rank] = if is_uniform(child) {
            slot(UNIFORM, child[0])
        } else {
            push_leaf(words, child)?
        };
    }

    offset_slot(NODE, node)
}

/// Appends the leaf over a child's 64 `cells` to `words`, and returns its
/// slot.
fn push_leaf(words: &mut Vec<u32>, cells: &[u32]) -> Result<u32> {
    let mut palette = Vec::new();

    let indices: Vec<u32> = cells
        .iter()
        .map(|&value| {
            let index = palette
                .iter()
                .position(|&entry| entry == value)
                .unwrap_or_else(|| {
                    palette.push(value);

                    palette.len() - 1
                });

            index as u32
        })
        .collect();

    let (tag, bits) = (LEAF..)
        .zip([1, 2, 4, 8])
        .find(|&(_, bits)| palette.len() <= 1 << bits)
        .expect("a leaf's 64 cells hold at most 64 values");

    let leaf = words.len();

    words.resize(leaf + 2 * bits as usize, 0);

    for (cell, index) in (0..).zip(indices) {
        let start: u32 = cell * bits;

        words[leaf + (start / 32) as usize] |= index << (start % 32);
    }

    words.extend(palette);

    offset_slot(tag, leaf)
}

#[cfg(test)]
mod tests {
    use crate::{
        Tree64, b1_scenes, painted_object,
        tree64::{EMPTY, LEAF, NODE, PAYLOAD, TAG_SHIFT, UNIFORM, raster, slot, tile_counts},
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn the_tree_gives_back_every_cell_of_each_scene() {
        for (name, scene) in b1_scenes() {
            for (object_id, object) in scene.iter_objects() {
                let tree = Tree64::new(object).unwrap();
                let bounds = object.bounds();

                for index in 0..bounds.x * bounds.y * bounds.z {
                    let voxel_id = U32Id::from_u32(index);
                    let position = object.voxel_position(voxel_id).unwrap();

                    assert_eq!(
                        tree.material(position),
                        object.voxel_material(voxel_id),
                        "{name}, object {object_id}, cell {position}"
                    );
                }

                assert_eq!(tree.material(bounds), None);
            }
        }
    }

    #[test]
    fn each_box_holds_only_empty_tiles() {
        for (name, scene) in b1_scenes() {
            for (object_id, object) in scene.iter_objects() {
                let tree = Tree64::new(object).unwrap();
                let tiles = tile_counts(object.bounds());
                let words = tree.words();
                let tile_ids = (0..tiles.x).flat_map(|x| {
                    (0..tiles.y).flat_map(move |y| (0..tiles.z).map(move |z| (x, y, z)))
                });

                for (x, y, z) in tile_ids {
                    let tile = TyVector3U32::new(x, y, z);
                    let tile_slot = words[raster(tiles, tile)];

                    if tile_slot >> TAG_SHIFT != EMPTY {
                        continue;
                    }

                    let reach = tile_slot & PAYLOAD;
                    let side = |shift: u32| reach >> shift & 15;
                    let low = tile - TyVector3U32::new(side(0), side(4), side(8));
                    let high = tile + TyVector3U32::new(side(12), side(16), side(20));

                    for bx in low.x..=high.x {
                        for by in low.y..=high.y {
                            for bz in low.z..=high.z {
                                let inner = words[raster(tiles, TyVector3U32::new(bx, by, bz))];

                                assert!(
                                    inner == slot(UNIFORM, 0) || inner >> TAG_SHIFT == EMPTY,
                                    "{name}, object {object_id}, tile {tile}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn an_empty_tile_reaches_its_empty_neighbors() {
        let material = U32Id::from_u32(0);

        // Two empty tiles, then a solid one.
        let object = painted_object("o", TyVector3U32::new(48, 16, 16), |p| {
            (p.x >= 32).then_some(material)
        });

        let tree = Tree64::new(&object).unwrap();

        assert_eq!(
            tree.words(),
            [slot(EMPTY, 1 << 12), slot(EMPTY, 1), slot(UNIFORM, 1)]
        );
        assert_eq!(tree.material(TyVector3U32::new(20, 3, 9)), None);
    }

    #[test]
    fn regions_collapse_and_leaves_size_their_palettes() {
        let material = U32Id::from_u32;

        // The first tile is solid. In the second, child 0 is solid, child 1
        // holds two values, child 2 holds five, and the rest are empty.
        let object = painted_object("o", TyVector3U32::new(32, 16, 16), |p| {
            match (p.x / 16, p.x % 16 / 4, p.y / 4, p.z / 4) {
                (0, ..) => Some(material(0)),
                (1, 0, 0, 0) => Some(material(1)),
                (1, 0, 0, 1) => (p.z % 2 == 0).then_some(material(0)),
                (1, 0, 0, 2) => Some(material((p.x % 4 * 2 + p.y % 2).min(4))),
                _ => None,
            }
        });

        let tree = Tree64::new(&object).unwrap();

        assert_eq!(
            tree.words()[..7],
            [
                1,
                slot(NODE, 2),
                0b111,
                0,
                2,
                slot(LEAF, 7),
                slot(LEAF + 2, 11)
            ]
        );
        assert_eq!(tree.words()[9..11], [1, 0]);
        assert_eq!(tree.words()[19..], [1, 2, 3, 4, 5]);

        for x in 0..32 {
            for y in 0..16 {
                for z in 0..16 {
                    let position = TyVector3U32::new(x, y, z);
                    let voxel_id = object.voxel_id(position).unwrap();

                    assert_eq!(tree.material(position), object.voxel_material(voxel_id));
                }
            }
        }
    }
}
