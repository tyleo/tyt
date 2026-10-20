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
        tree64::{LEAF, NODE, slot},
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
