use crate::{Format, Result};
use std::path::Path;
use voxsmith::to_mvox_bytes;

/// Converts the voxel file at `input` into a MagicaVoxel `.vox` file at
/// `output`, round-tripping through voxcore: the input is loaded into a
/// [`VoxState`](voxcore::VoxState), then encoded back to `.vox` bytes.
pub(crate) fn to_mvox(input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
    let state = super::load_state::load_state(input, from)?;
    let bytes = to_mvox_bytes(&state)?;
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
}
