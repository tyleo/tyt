/// A `.vox` `ROTATION`: a row-major signed permutation matrix packed into one
/// byte. Each row has exactly one non-zero entry, `+1` or `-1`; the byte stores,
/// for each row, the column of its non-zero entry and that entry's sign.
///
/// Bits `0-1` and `2-3` hold the non-zero column of rows `0` and `1` (row `2`
/// takes the remaining column); bits `4`, `5`, `6` hold the sign of rows `0`,
/// `1`, `2` (`1` = negative).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MVoxRotation(pub u8);

impl MVoxRotation {
    /// The identity rotation (byte `0x04`): rows `0`, `1`, `2` keep columns `0`,
    /// `1`, `2`, all positive.
    pub const IDENTITY: MVoxRotation = MVoxRotation(0b0000_0100);

    /// Decodes the byte into a row-major 3x3 matrix whose entries are `-1`, `0`,
    /// or `+1`. A byte that is not a valid signed permutation leaves the
    /// offending rows zero rather than panicking.
    pub fn to_matrix(self) -> [[i8; 3]; 3] {
        let byte = self.0;
        let col0 = (byte & 0b11) as usize;
        let col1 = ((byte >> 2) & 0b11) as usize;
        let sign = |bit: u8| if (byte >> bit) & 1 == 1 { -1i8 } else { 1 };
        let mut matrix = [[0i8; 3]; 3];
        if col0 < 3 {
            matrix[0][col0] = sign(4);
        }
        if col1 < 3 {
            matrix[1][col1] = sign(5);
        }
        // Row 2's column is the one the first two rows leave free.
        if col0 < 3 && col1 < 3 && col0 != col1 {
            matrix[2][3 - col0 - col1] = sign(6);
        }
        matrix
    }

    /// Encodes a row-major 3x3 matrix into a rotation byte, or `None` if it is
    /// not a signed permutation matrix (exactly one `+1` or `-1` per row and per
    /// column, every other entry `0`).
    pub fn from_matrix(matrix: [[i8; 3]; 3]) -> Option<MVoxRotation> {
        let mut byte = 0u8;
        let mut columns_used = [false; 3];
        for (row, entries) in matrix.iter().enumerate() {
            let mut nonzero = None;
            for (col, &value) in entries.iter().enumerate() {
                match value {
                    0 => {}
                    1 | -1 if nonzero.is_none() => nonzero = Some((col, value)),
                    _ => return None,
                }
            }
            let (col, value) = nonzero?;
            if columns_used[col] {
                return None;
            }
            columns_used[col] = true;
            if row < 2 {
                byte |= (col as u8) << (row * 2);
            }
            if value == -1 {
                byte |= 1 << (4 + row as u8);
            }
        }
        Some(MVoxRotation(byte))
    }
}

impl Default for MVoxRotation {
    fn default() -> Self {
        MVoxRotation::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use crate::MVoxRotation;
    use std::collections::BTreeSet;

    #[test]
    fn identity_round_trips() {
        let identity = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
        assert_eq!(MVoxRotation::IDENTITY.to_matrix(), identity);
        assert_eq!(
            MVoxRotation::from_matrix(identity),
            Some(MVoxRotation::IDENTITY)
        );
    }

    #[test]
    fn matches_spec_example() {
        // From the format spec: this matrix encodes to byte 105.
        let matrix = [[0, 1, 0], [0, 0, -1], [-1, 0, 0]];
        let rotation = MVoxRotation(105);
        assert_eq!(rotation.to_matrix(), matrix);
        assert_eq!(MVoxRotation::from_matrix(matrix), Some(rotation));
    }

    #[test]
    fn every_valid_rotation_round_trips() {
        // Bit 7 is unused, so each matrix has two encodings; collect the
        // distinct matrices to count them. A 3x3 signed permutation has 3!
        // column orderings times 2^3 sign choices = 48.
        let mut matrices = BTreeSet::new();
        for byte in 0u8..=255 {
            let matrix = MVoxRotation(byte).to_matrix();
            if let Some(reencoded) = MVoxRotation::from_matrix(matrix) {
                assert_eq!(reencoded.to_matrix(), matrix, "byte {byte}");
                matrices.insert(matrix);
            }
        }
        assert_eq!(matrices.len(), 48);
    }

    #[test]
    fn rejects_non_permutation() {
        assert_eq!(
            MVoxRotation::from_matrix([[1, 1, 0], [0, 0, 1], [0, 0, 0]]),
            None
        );
        assert_eq!(
            MVoxRotation::from_matrix([[2, 0, 0], [0, 1, 0], [0, 0, 1]]),
            None
        );
    }
}
