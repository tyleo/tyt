use crate::{Format, Result};
use std::path::Path;
use voxsmith::to_qbcl_bytes;

/// Converts the voxel file at `input` into a Qubicle `.qbcl` file at `output`,
/// round-tripping through voxcore: the input is loaded into a
/// [`VoxState`](voxcore::VoxState), then encoded back to `.qbcl` bytes. Requires
/// the source to carry the `qubicle-qbcl` provenance voxsmith writes.
pub(crate) fn to_qbcl(input: &Path, from: Option<Format>, output: &Path) -> Result<()> {
    let state = super::load_state::load_state(input, from)?;
    let bytes = to_qbcl_bytes(&state)?;
    tyt_injection::write_file(output, &bytes)?;
    Ok(())
}
