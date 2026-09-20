/// The `u32` id of the record table entry at `index`.
pub(crate) fn table_index(index: usize) -> u32 {
    u32::try_from(index).expect("a record table is shorter than u32::MAX")
}
