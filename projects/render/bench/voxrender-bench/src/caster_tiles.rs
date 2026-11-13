use crate::{Error, Result};
use ty_math::TyVector3U32;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The cells of a tile.
const CELLS: usize = (TILE * TILE * TILE) as usize;

/// An object's cells by tile, as `lit_fill.wgsl`'s shadow rays read them.
/// The words grow with the object's tiles that hold a cell, where a word
/// for each cell of its grid grows with the grid.
///
/// The words start with a word for each tile of the grid. The tile at
/// `(x, y, z)` among the tiles takes word `(x * counts.y + y) * counts.z +
/// z`, where `counts` holds the tiles along each axis. A tile with no live
/// cell holds zero. Any other holds where its words start among the
/// object's, with [`KEYED`](Self::KEYED) set where it holds a cell that
/// passes light.
///
/// A tile's words start with [`BIT_WORDS`](Self::BIT_WORDS) words of a bit
/// for each cell, set where the cell is opaque. Cell `(x, y, z)` of the
/// tile has index `(x * 16 + y) * 16 + z`, and takes bit `index % 32` of
/// word `index / 32`. A keyed tile goes on with 16 bits for each cell, two
/// cells a word from the low bits up. They hold the material id plus one
/// of a cell that passes light, and zero for any other cell.
pub struct CasterTiles {
    pub words: Vec<u32>,
}

impl CasterTiles {
    /// The bit of a tile's word that marks a tile whose keys follow its
    /// bits.
    pub const KEYED: u32 = 1 << 31;

    /// The words of a tile's bits.
    pub const BIT_WORDS: usize = CELLS / 32;

    /// The tiles of a grid of `bounds`. Errors if a material that passes
    /// light has an id over 65534.
    ///
    /// # Arguments
    ///
    /// 1. `cells`: the grid's cells in raster order, 0 when empty, else the
    ///    material id plus one
    /// 2. `opaque`: whether a material id passes no light
    pub fn new(bounds: TyVector3U32, cells: &[u32], opaque: &dyn Fn(u32) -> bool) -> Result<Self> {
        let counts = (bounds + TILE - 1) / TILE;
        let mut words = vec![0; (counts.x * counts.y * counts.z) as usize];
        let mut bits = [0u32; Self::BIT_WORDS];
        let mut keys = [0u32; CELLS / 2];

        // Whether each material id met so far is opaque.
        let mut known: Vec<Option<bool>> = Vec::new();

        for tile in 0..words.len() as u32 {
            let low = TyVector3U32::new(
                tile / (counts.y * counts.z),
                tile / counts.z % counts.y,
                tile % counts.z,
            ) * TILE;
            let high = (low + TILE).min(bounds);
            let mut live = false;
            let mut keyed = false;

            bits.fill(0);
            keys.fill(0);

            for x in low.x..high.x {
                for y in low.y..high.y {
                    let row = ((x * bounds.y + y) * bounds.z) as usize;

                    for z in low.z..high.z {
                        let Some(id) = cells[row + z as usize].checked_sub(1) else {
                            continue;
                        };

                        if known.len() <= id as usize {
                            known.resize(id as usize + 1, None);
                        }

                        let blocks = *known[id as usize].get_or_insert_with(|| opaque(id));
                        let index = (((x - low.x) * TILE + y - low.y) * TILE + z - low.z) as usize;

                        live = true;

                        if blocks {
                            bits[index / 32] |= 1 << (index % 32);
                        } else {
                            if id >= 0xffff {
                                return Err(Error::PassingMaterial { id });
                            }

                            keys[index / 2] |= (id + 1) << (index % 2 * 16);
                            keyed = true;
                        }
                    }
                }
            }

            if !live {
                continue;
            }

            let first = u32::try_from(words.len())
                .ok()
                .filter(|&first| first < Self::KEYED)
                .expect("an object's caster words number under 2^31");

            words[tile as usize] = first | if keyed { Self::KEYED } else { 0 };
            words.extend(bits);

            if keyed {
                words.extend(keys);
            }
        }

        Ok(CasterTiles { words })
    }
}

#[cfg(test)]
mod tests {
    use crate::CasterTiles;
    use ty_math::TyVector3U32;

    #[test]
    fn a_tile_holds_its_opaque_cells_as_bits_and_its_passing_cells_as_keys() {
        let bounds = TyVector3U32::new(40, 16, 16);
        let mut cells = vec![0; 40 * 16 * 16];

        // An opaque cell in the first tile. The second tile is empty. In
        // the third, a cell of material 4, which passes light, beside an
        // opaque cell.
        cells[(3 * 16 + 2) * 16 + 1] = 1;
        cells[(33 * 16 + 5) * 16 + 7] = 5;
        cells[(33 * 16 + 5) * 16 + 8] = 1;

        let tiles = CasterTiles::new(bounds, &cells, &|id| id == 0).unwrap();
        let words = &tiles.words;

        assert_eq!(words[..3], [3, 0, (3 + 128) | CasterTiles::KEYED]);
        assert_eq!(words.len(), 3 + 128 + 128 + 2048);

        let index = (3 * 16 + 2) * 16 + 1;

        assert_eq!(words[3 + index / 32], 1 << (index % 32));

        let third = 3 + 128;
        let index = (16 + 5) * 16 + 7;

        assert_eq!(words[third + (index + 1) / 32], 1 << ((index + 1) % 32));
        assert_eq!(words[third + 128 + index / 2], 5 << 16);
    }

    #[test]
    fn a_passing_material_past_the_keys_bits_errors() {
        let cells = [0x1_0000];

        assert!(CasterTiles::new(TyVector3U32::splat(1), &cells, &|_| false).is_err());
        assert!(CasterTiles::new(TyVector3U32::splat(1), &cells, &|_| true).is_ok());
    }
}
