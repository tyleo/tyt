use crate::{Result, invalid, voxel_count};

/// Checks that a matrix of `size` holds `len` voxels, one per cell.
pub fn check_voxel_count(size: [u32; 3], len: usize) -> Result<()> {
    let expected = voxel_count(size)?;
    if len != expected {
        return Err(invalid(format!(
            "matrix size {size:?} has {expected} cells but its grid holds {len} voxels"
        )));
    }

    Ok(())
}
