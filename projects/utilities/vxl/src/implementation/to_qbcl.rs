use crate::{Format, Result, implementation};
use std::{fs, path::Path};
use voxsmith::to_qbcl_bytes;

/// Converts the voxel file at `input` into a Qubicle `.qbcl` file at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), then encoded back to `.qbcl` bytes.
pub fn to_qbcl(input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
    let state = implementation::load_state_qbcl(input, from)?;
    let bytes = to_qbcl_bytes(&state)?;
    fs::write(output, &bytes)?;
    Ok(())
}
