use crate::{Format, Result, implementation};
use std::path::Path;
use voxsmith::to_goxl_bytes;

/// Converts the voxel file at `input` into a Goxel `.gox` file at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), then encoded back to `.gox` bytes.
pub fn to_goxl(input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let bytes = to_goxl_bytes(&state)?;
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
}
