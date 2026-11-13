use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// One object's cells, a word for each cell of its grid. A loader holds
/// one object's grid at a time and keeps the object's tiles in its place.
pub struct CellGrid {
    pub bounds: TyVector3U32,

    /// Each cell in raster order, `(x * bounds.y + y) * bounds.z + z`: 0
    /// when empty, else the place of its material among a scene's plus one.
    pub cells: Vec<u32>,
}

impl CellGrid {
    /// The cells of `object`, where `place` gives a material id's place
    /// among a scene's materials.
    pub fn new(object: &RenderObject, place: &dyn Fn(u32) -> u32) -> Self {
        let mut cells = dense_cells(object);

        for cell in cells.iter_mut().filter(|cell| **cell != 0) {
            *cell = place(*cell - 1) + 1;
        }

        CellGrid {
            bounds: object.bounds(),
            cells,
        }
    }

    /// The grid with `before` empty cells ahead of its first cell along
    /// each axis.
    pub fn padded(&self, before: TyVector3U32) -> Self {
        let bounds = self.bounds + before;
        let mut cells = vec![0; (bounds.x * bounds.y * bounds.z) as usize];
        let row = self.bounds.z as usize;

        for x in 0..self.bounds.x {
            for y in 0..self.bounds.y {
                let from = ((x * self.bounds.y + y) * self.bounds.z) as usize;
                let to =
                    (((x + before.x) * bounds.y + y + before.y) * bounds.z + before.z) as usize;

                cells[to..to + row].copy_from_slice(&self.cells[from..from + row]);
            }
        }

        CellGrid { bounds, cells }
    }

    /// The grid at half its cells along each axis. A cell stands for the
    /// eight it covers, and is live where one of them is. It takes the
    /// material most of its live cells share, and a tie goes to the
    /// earliest of them in raster order. A cell past the grid counts as
    /// empty.
    pub fn halved(&self) -> Self {
        let bounds = (self.bounds + 1) / 2;
        let mut cells = Vec::with_capacity((bounds.x * bounds.y * bounds.z) as usize);

        for x in 0..bounds.x {
            for y in 0..bounds.y {
                for z in 0..bounds.z {
                    // The live cells' values in raster order.
                    let mut live = [0; 8];
                    let mut count = 0;

                    for corner in 0..8 {
                        let at = TyVector3U32::new(
                            2 * x + (corner >> 2),
                            2 * y + (corner >> 1 & 1),
                            2 * z + (corner & 1),
                        );

                        if at.cmplt(self.bounds).all() {
                            let index = (at.x * self.bounds.y + at.y) * self.bounds.z + at.z;
                            let value = self.cells[index as usize];

                            if value != 0 {
                                live[count] = value;
                                count += 1;
                            }
                        }
                    }

                    let live = &live[..count];
                    let shared = |value: &u32| live.iter().filter(|&other| other == value).count();

                    // The first of the values that the most cells share.
                    let most = live
                        .iter()
                        .rev()
                        .max_by_key(|&value| shared(value))
                        .copied();

                    cells.push(most.unwrap_or(0));
                }
            }
        }

        CellGrid { bounds, cells }
    }
}

#[cfg(test)]
mod tests {
    use crate::CellGrid;
    use ty_math::TyVector3U32;

    #[test]
    fn a_halved_cell_is_live_where_one_of_its_eight_is() {
        // Three cells along x, two deep along z: materials 4 and 7 in the
        // first two columns, and one cell of 9 in the third.
        let grid = CellGrid {
            bounds: TyVector3U32::new(3, 1, 2),
            cells: vec![5, 8, 8, 0, 0, 10],
        };

        let halved = grid.halved();

        assert_eq!(halved.bounds, TyVector3U32::new(2, 1, 1));

        // Material 7 holds two of the first cell's three live cells. The
        // second cell reaches past the grid and holds one live cell.
        assert_eq!(halved.cells, [8, 10]);
    }

    #[test]
    fn a_tie_goes_to_the_earliest_cell() {
        let grid = CellGrid {
            bounds: TyVector3U32::new(1, 1, 2),
            cells: vec![3, 2],
        };

        assert_eq!(grid.halved().cells, [3]);
    }

    #[test]
    fn a_padded_grid_holds_its_cells_past_the_padding() {
        let grid = CellGrid {
            bounds: TyVector3U32::new(1, 2, 1),
            cells: vec![6, 7],
        };

        let padded = grid.padded(TyVector3U32::new(1, 0, 1));

        assert_eq!(padded.bounds, TyVector3U32::new(2, 2, 2));
        assert_eq!(padded.cells, [0, 0, 0, 0, 0, 6, 0, 7]);
    }
}
