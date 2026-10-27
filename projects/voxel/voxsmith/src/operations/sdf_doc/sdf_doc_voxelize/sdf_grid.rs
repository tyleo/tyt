use crate::operations::sdf_doc::{SdfCell, SdfStepRecord};
use branded_id::U32Id;
use sdfcore::BSdfObject;
use ty_math::{TyVector3I32, TyVector3U32};

/// One object's grid after the object's steps ran.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfGrid {
    /// The part's object.
    pub object_id: U32Id<BSdfObject>,

    /// The index of the grid's least cell along each axis. The cell
    /// `[i, j, k]` spans from `[i, j, k]` to `[i + 1, j + 1, k + 1]` times
    /// the voxel size.
    pub min: TyVector3I32,

    /// The cells along each axis.
    pub size: TyVector3U32,

    /// Each cell in a raster with x outermost and z innermost.
    pub cells: Vec<SdfCell>,

    /// What each step of the object's list wrote, in list order.
    pub steps: Vec<SdfStepRecord>,
}

impl SdfGrid {
    /// The cell at the lattice index `cell`, or `None` outside the grid.
    pub fn cell(&self, cell: TyVector3I32) -> Option<&SdfCell> {
        self.index(cell).map(|index| &self.cells[index])
    }

    /// The lattice index of the cell at position `index` in `cells`.
    pub fn lattice_index(&self, index: usize) -> TyVector3I32 {
        let size = self.size.as_ivec3();
        let index = index as i32;

        self.min
            + TyVector3I32::new(
                index / (size.y * size.z),
                index / size.z % size.y,
                index % size.z,
            )
    }

    /// The position in `cells` of the cell at the lattice index `cell`, or
    /// `None` outside the grid.
    pub fn index(&self, cell: TyVector3I32) -> Option<usize> {
        let offsets = [0, 1, 2].map(|axis| i64::from(cell[axis]) - i64::from(self.min[axis]));
        let sizes = [0, 1, 2].map(|axis| i64::from(self.size[axis]));

        if (0..3).any(|axis| offsets[axis] < 0 || offsets[axis] >= sizes[axis]) {
            return None;
        }

        Some(((offsets[0] * sizes[1] + offsets[1]) * sizes[2] + offsets[2]) as usize)
    }
}
