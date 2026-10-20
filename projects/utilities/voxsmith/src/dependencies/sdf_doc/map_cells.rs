use ty_math::TyVector3I32;

/// Maps a voxelizer step's cells to their results. An implementation picks
/// how the cells spread across threads.
pub trait MapCells {
    /// `map_cell` applied to each of `cells`, in order.
    fn map_cells<T, F>(&self, cells: &[TyVector3I32], map_cell: F) -> Vec<T>
    where
        T: Send,
        F: Fn(TyVector3I32) -> T + Sync;
}
