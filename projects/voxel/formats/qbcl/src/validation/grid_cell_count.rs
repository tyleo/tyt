/// The cell count a `[x, y, z]` grid size needs, or `None` when the count
/// overflows `usize`.
pub fn grid_cell_count(size: [u32; 3]) -> Option<usize> {
    (size[0] as usize)
        .checked_mul(size[1] as usize)
        .and_then(|xy| xy.checked_mul(size[2] as usize))
}

#[cfg(test)]
mod tests {
    use crate::validation::grid_cell_count;

    #[test]
    fn multiplies_the_three_axes() {
        assert_eq!(grid_cell_count([3, 2, 4]), Some(24));
    }

    #[test]
    fn returns_none_on_overflow() {
        assert_eq!(grid_cell_count([u32::MAX, u32::MAX, u32::MAX]), None);
    }
}
