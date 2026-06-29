use crate::{Format, Result, implementation};
use std::path::Path;
use voxsmith::to_mvox_bytes;

/// Converts the voxel file at `input` into a MagicaVoxel `.vox` file at
/// `output`, round-tripping through voxcore: the input is loaded into a
/// [`VoxMain`](voxcore::VoxMain), then encoded back to `.vox` bytes.
pub fn to_mvox(input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let bytes = to_mvox_bytes(&state)?;
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
}
