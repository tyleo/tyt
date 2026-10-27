/// Maps a render's rows to their results. An implementation picks how the
/// rows spread across threads.
pub trait MapRows {
    /// `map_row` applied to each row below `height`, in row order.
    fn map_rows<T, F>(&self, height: u32, map_row: F) -> Vec<T>
    where
        T: Send,
        F: Fn(u32) -> T + Sync;
}
