use crate::SurfaceGrid;
use ty_math::TyVector3U32;

/// A grid whose cells each carry a material and whether it is opaque.
pub struct MaterialGrid {
    bounds: TyVector3U32,

    /// Each cell in raster order, `(material, opaque)` where solid.
    cells: Vec<Option<(u32, bool)>>,
}

impl MaterialGrid {
    /// A grid of `bounds` holding each of `cells` as `(position, material,
    /// opaque)`.
    pub fn new(bounds: [u32; 3], cells: &[([u32; 3], u32, bool)]) -> Self {
        let bounds = TyVector3U32::from_array(bounds);
        let mut grid = MaterialGrid {
            bounds,
            cells: vec![None; (bounds.x * bounds.y * bounds.z) as usize],
        };

        for &(position, material, opaque) in cells {
            let index = grid.index(TyVector3U32::from_array(position));
            grid.cells[index] = Some((material, opaque));
        }

        grid
    }

    fn index(&self, position: TyVector3U32) -> usize {
        ((position.x * self.bounds.y + position.y) * self.bounds.z + position.z) as usize
    }

    fn material(&self, cell: usize) -> (u32, bool) {
        self.cells[cell].expect("a handle names a solid cell")
    }
}

impl SurfaceGrid for MaterialGrid {
    type Cell = usize;

    fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        let index = self.index(position);

        self.cells[index].map(|_| index)
    }

    fn is_opaque(&self, cell: Self::Cell) -> bool {
        self.material(cell).1
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.material(cell).0 == self.material(other).0
    }
}
