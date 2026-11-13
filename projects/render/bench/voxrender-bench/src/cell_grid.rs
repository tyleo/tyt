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
}
