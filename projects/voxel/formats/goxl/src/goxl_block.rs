use crate::GoxlVoxel;

/// A `BL16` voxel block: a dense `16 x 16 x 16` grid of [`GoxlVoxel`] cells.
/// Layers reference blocks by index and tile them across the scene, so an
/// identical block is stored once and shared.
///
/// [`voxels`](Self::voxels) holds all [`SIZE`](Self::SIZE)`^3 == 4096` cells in
/// storage order (X fastest, then Y, then Z), so `(x, y, z)` is at index
/// `x + 16 * y + 256 * z`. Empty cells are stored too (alpha `0`); use
/// [`voxel`](Self::voxel) to index by coordinate.
///
/// On disk the grid is a `64 x 64` `RGBA` PNG whose 4096 pixels are exactly
/// these cells in the same order; `goxl-codec` does that conversion.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoxlBlock {
    /// Dense voxel grid, [`SIZE`](Self::SIZE)`^3` cells in storage order.
    pub voxels: Vec<GoxlVoxel>,
}

impl GoxlBlock {
    /// The edge length of a block, in voxels: `16`.
    pub const SIZE: u32 = 16;

    /// The cell at `(x, y, z)`, or `None` if it is outside the
    /// [`SIZE`](Self::SIZE)-cubed grid or the grid is not fully populated.
    pub fn voxel(&self, x: u32, y: u32, z: u32) -> Option<GoxlVoxel> {
        if x >= Self::SIZE || y >= Self::SIZE || z >= Self::SIZE {
            return None;
        }
        let size = Self::SIZE as usize;
        let index = x as usize + size * (y as usize + size * z as usize);
        self.voxels.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::{GoxlBlock, GoxlVoxel};

    #[test]
    fn voxel_indexes_in_x_y_z_order() {
        let size = GoxlBlock::SIZE;
        // Build the full grid in storage order, each cell tagged with its
        // coordinate (coordinates `0..16` fit in a `u8` channel).
        let mut voxels = Vec::new();
        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    voxels.push(GoxlVoxel::new(x as u8, y as u8, z as u8));
                }
            }
        }
        let block = GoxlBlock { voxels };

        for z in 0..size {
            for y in 0..size {
                for x in 0..size {
                    let voxel = block.voxel(x, y, z).unwrap();
                    assert_eq!((voxel.r, voxel.g, voxel.b), (x as u8, y as u8, z as u8));
                }
            }
        }
        assert_eq!(block.voxel(size, 0, 0), None);
        assert_eq!(block.voxel(0, 0, size), None);
        assert_eq!(GoxlBlock::default().voxel(0, 0, 0), None);
    }
}
