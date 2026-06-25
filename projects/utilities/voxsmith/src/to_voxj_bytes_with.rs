use crate::{Result, to_voxj_file_with};
use voxcore::VoxState;
use voxj_codec::{PositionEncoding, SampleEncoding, to_voxj_file_bytes};

/// Writes a [`VoxState`] to compact `.voxj` JSON bytes with fixed `position` and
/// `sample` block encodings applied to every object. For the smallest-per-object
/// search instead, see [`to_voxj_bytes`](crate::to_voxj_bytes).
pub fn to_voxj_bytes_with(
    state: &VoxState,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<Vec<u8>> {
    let file = to_voxj_file_with(state, position, sample)?;
    Ok(to_voxj_file_bytes(&file)?)
}
