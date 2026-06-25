use crate::qbcl::QbclVoxel;

/// The voxel grid of a `.qbcl` matrix or compound node: a dense grid of
/// [`size`](Self::size) cells placed and pivoted in the scene.
///
/// [`voxels`](Self::voxels) holds all `size[0] * size[1] * size[2]` cells in
/// storage order (X outermost, then Z, then Y), so `(x, y, z)` is at index
/// `y + size[1] * (z + size[2] * x)`. Empty cells are stored too (mask `0`); use
/// [`voxel`](Self::voxel) to index by coordinate. The node's name and editor
/// flags live on the enclosing [`QbclNode`](crate::qbcl::QbclNode), not here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QbclMatrix {
    /// `[x, y, z]` grid size in voxels.
    pub size: [u32; 3],

    /// `[x, y, z]` position in the scene.
    pub position: [i32; 3],

    /// `[x, y, z]` pivot, in voxel coordinates.
    pub pivot: [f32; 3],

    /// Dense voxel grid, `size[0] * size[1] * size[2]` cells in storage order.
    pub voxels: Vec<QbclVoxel>,
}

impl QbclMatrix {
    /// The cell at `(x, y, z)`, or `None` if it is outside [`size`](Self::size)
    /// or the grid is not fully populated.
    pub fn voxel(&self, x: u32, y: u32, z: u32) -> Option<QbclVoxel> {
        let [size_x, size_y, size_z] = self.size;
        if x >= size_x || y >= size_y || z >= size_z {
            return None;
        }
        let index = y as usize + size_y as usize * (z as usize + size_z as usize * x as usize);
        self.voxels.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::qbcl::{QbclMatrix, QbclVoxel};

    #[test]
    fn voxel_indexes_in_x_z_y_order() {
        let size = [3u32, 2, 2];
        let [size_x, size_y, size_z] = size;
        // Build the grid in storage order, each cell tagged with its coordinate.
        let mut voxels = Vec::new();
        for x in 0..size_x {
            for z in 0..size_z {
                for y in 0..size_y {
                    voxels.push(QbclVoxel::new(x as u8, y as u8, z as u8, 0x7e));
                }
            }
        }
        let matrix = QbclMatrix {
            size,
            voxels,
            ..Default::default()
        };

        for x in 0..size_x {
            for z in 0..size_z {
                for y in 0..size_y {
                    let voxel = matrix.voxel(x, y, z).unwrap();
                    assert_eq!((voxel.r, voxel.g, voxel.b), (x as u8, y as u8, z as u8));
                }
            }
        }
        assert_eq!(matrix.voxel(3, 0, 0), None);
        assert_eq!(matrix.voxel(0, 2, 0), None);
    }
}
