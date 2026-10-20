use crate::dependencies::{DependenciesImpl, sdf_doc::MapCells};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use ty_math::TyVector3I32;

impl MapCells for DependenciesImpl {
    fn map_cells<T, F>(&self, cells: &[TyVector3I32], map_cell: F) -> Vec<T>
    where
        T: Send,
        F: Fn(TyVector3I32) -> T + Sync,
    {
        cells.par_iter().map(|&cell| map_cell(cell)).collect()
    }
}
