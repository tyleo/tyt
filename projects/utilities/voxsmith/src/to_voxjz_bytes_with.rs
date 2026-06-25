use crate::{Result, to_voxj_file_with};
use voxcore::VoxState;
use voxj_codec::{PositionEncoding, SampleEncoding, to_voxjz_file_bytes};

/// Writes a [`VoxState`] to a `.voxjz` zip archive holding one compact `.voxj`
/// member, with fixed `position` and `sample` block encodings applied to every
/// object. For the smallest-per-object search instead, see
/// [`to_voxjz_bytes`](crate::to_voxjz_bytes).
pub fn to_voxjz_bytes_with(
    state: &VoxState,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<Vec<u8>> {
    let file = to_voxj_file_with(state, position, sample)?;
    Ok(to_voxjz_file_bytes(&file)?)
}
