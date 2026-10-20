use crate::MapRows;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// The dependencies the `impl` feature provides. `map_rows` spreads the rows
/// across threads.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl MapRows for DependenciesImpl {
    fn map_rows<T, F>(&self, height: u32, map_row: F) -> Vec<T>
    where
        T: Send,
        F: Fn(u32) -> T + Sync,
    {
        (0..height).into_par_iter().map(&map_row).collect()
    }
}
