use crate::MapRows;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// The dependencies over `rayon`. `map_rows` spreads the rows across
/// `rayon`'s global thread pool.
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
