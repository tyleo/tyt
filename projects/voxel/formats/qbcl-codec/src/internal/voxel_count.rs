use crate::{Result, invalid};

/// The number of cells in a `size`, or an error if it overflows `usize`.
pub fn voxel_count(size: [u32; 3]) -> Result<usize> {
    (size[0] as usize)
        .checked_mul(size[1] as usize)
        .and_then(|xy| xy.checked_mul(size[2] as usize))
        .ok_or_else(|| {
            invalid(format!(
                "matrix size {size:?} overflows the addressable range"
            ))
        })
}
