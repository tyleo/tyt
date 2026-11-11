use crate::live_boxes;
use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// An object's non-empty tiles as a mask march reads them.
pub struct TileMasks {
    /// Each tile's low live cell and the corner past its high live cell, in
    /// raster order.
    pub boxes: Vec<(TyVector3U32, TyVector3U32)>,

    /// [`WORDS`](Self::WORDS) words per tile, in the boxes' order. A tile's
    /// cell `(x, y, z)` is live where bit `y % 2 * 16 + z` of the tile's
    /// word `x * 8 + y / 2` is set.
    pub words: Vec<u32>,
}

impl TileMasks {
    /// The words of one tile's bits.
    pub const WORDS: u32 = TILE * TILE / 2;

    /// The masks of `object`'s non-empty tiles.
    pub fn new(object: &RenderObject) -> Self {
        let bounds = object.bounds();
        let cells = dense_cells(object);
        let boxes = live_boxes(object, TILE);
        let mut words = vec![0; boxes.len() * Self::WORDS as usize];

        for (tile, &(low, high)) in boxes.iter().enumerate() {
            let first = tile * Self::WORDS as usize;

            for x in low.x..high.x {
                for y in low.y..high.y {
                    for z in low.z..high.z {
                        if cells[((x * bounds.y + y) * bounds.z + z) as usize] != 0 {
                            let at = TyVector3U32::new(x, y, z) % TILE;

                            words[first + (at.x * 8 + at.y / 2) as usize] |=
                                1 << (at.y % 2 * 16 + at.z);
                        }
                    }
                }
            }
        }

        TileMasks { boxes, words }
    }
}

#[cfg(test)]
mod tests {
    use crate::{TileMasks, b1_scenes};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn the_masks_give_back_every_cell_of_each_scene() {
        for (name, scene) in b1_scenes() {
            for (object_id, object) in scene.iter_objects() {
                let masks = TileMasks::new(object);
                let bounds = object.bounds();
                let mut live = 0;

                for (tile, &(low, _)) in masks.boxes.iter().enumerate() {
                    let corner = low / 16 * 16;

                    for bit in 0..4096 {
                        let at = TyVector3U32::new(bit / 256, bit / 16 % 16, bit % 16);
                        let word = masks.words[tile * 128 + (at.x * 8 + at.y / 2) as usize];
                        let set = word >> (at.y % 2 * 16 + at.z) & 1 == 1;

                        let cell = corner + at;
                        let filled = cell.cmplt(bounds).all()
                            && object
                                .voxel_id(cell)
                                .is_some_and(|voxel_id| object.is_live(voxel_id));

                        assert_eq!(set, filled, "{name}, object {object_id}, cell {cell}");

                        live += usize::from(set);
                    }
                }

                let cells = (0..bounds.x * bounds.y * bounds.z)
                    .filter(|&index| object.is_live(U32Id::from_u32(index)))
                    .count();

                assert_eq!(live, cells, "{name}, object {object_id}");
            }
        }
    }
}
