use crate::TyMatrix4x4F64;

/// The tyt-specific matrix operations glam does not name directly, carried on
/// [`TyMatrix4x4F64`] so callers keep `m.foo()` with `use ty_math::TyMatrix4x4Ext`.
pub trait TyMatrix4x4Ext {
    /// The element at `row` and `col`.
    ///
    /// # Panics
    /// Panics when `row` or `col` is not in `0..4`.
    fn get(&self, row: usize, col: usize) -> f64;
}

impl TyMatrix4x4Ext for TyMatrix4x4F64 {
    fn get(&self, row: usize, col: usize) -> f64 {
        self.col(col)[row]
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyMatrix4x4Ext, TyMatrix4x4F64};

    #[test]
    fn get_reads_row_and_column() {
        // from_cols_array_2d takes columns, so arrays[col][row].
        let matrix = TyMatrix4x4F64::from_cols_array_2d(&[
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        assert_eq!(matrix.get(0, 0), 1.0);
        assert_eq!(matrix.get(1, 0), 2.0);
        assert_eq!(matrix.get(0, 3), 13.0);
        assert_eq!(matrix.get(3, 3), 16.0);
    }
}
