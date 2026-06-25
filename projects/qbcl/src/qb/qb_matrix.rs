use crate::qb::QbVoxel;

/// A named `.qb` voxel matrix: a dense grid of [`size`](Self::size) cells at
/// [`position`](Self::position).
///
/// [`voxels`](Self::voxels) holds all `size[0] * size[1] * size[2]` cells in
/// storage order (Z outermost, then Y, then X), so `(x, y, z)` is at index
/// `x + size[0] * (y + size[1] * z)`. Empty cells are stored too (visibility
/// `0`); use [`voxel`](Self::voxel) to index by coordinate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QbMatrix {
    /// Matrix name.
    pub name: String,

    /// `[x, y, z]` grid size in voxels.
    pub size: [u32; 3],

    /// `[x, y, z]` position in the scene.
    pub position: [i32; 3],

    /// Dense voxel grid, `size[0] * size[1] * size[2]` cells in storage order.
    pub voxels: Vec<QbVoxel>,
}

impl QbMatrix {
    /// The cell at `(x, y, z)`, or `None` if it is outside [`size`](Self::size)
    /// or the grid is not fully populated.
    pub fn voxel(&self, x: u32, y: u32, z: u32) -> Option<QbVoxel> {
        let [size_x, size_y, size_z] = self.size;
        if x >= size_x || y >= size_y || z >= size_z {
            return None;
        }
        let index = x as usize + size_x as usize * (y as usize + size_y as usize * z as usize);
        self.voxels.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::qb::{QbMatrix, QbVoxel};

    #[test]
    fn voxel_indexes_in_z_y_x_order() {
        let size = [3u32, 2, 2];
        let [size_x, size_y, size_z] = size;
        // Build the grid in storage order, each cell tagged with its coordinate.
        let mut voxels = Vec::new();
        for z in 0..size_z {
            for y in 0..size_y {
                for x in 0..size_x {
                    voxels.push(QbVoxel::new(x as u8, y as u8, z as u8));
                }
            }
        }
        let matrix = QbMatrix {
            name: String::new(),
            size,
            position: [0, 0, 0],
            voxels,
        };

        for z in 0..size_z {
            for y in 0..size_y {
                for x in 0..size_x {
                    let voxel = matrix.voxel(x, y, z).unwrap();
                    assert_eq!((voxel.r, voxel.g, voxel.b), (x as u8, y as u8, z as u8));
                }
            }
        }
        assert_eq!(matrix.voxel(3, 0, 0), None);
        assert_eq!(matrix.voxel(0, 2, 0), None);
    }
}
