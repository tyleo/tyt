use crate::{Result, VoxjFileBuilder};
use voxcore::VoxMain;
use voxj_codec::{PositionEncoding, SampleEncoding, to_voxjz_file_bytes};

/// Writes a [`VoxMain`] to a `.voxjz` zip archive holding one compact `.voxj`
/// member, with fixed `position` and `sample` block encodings applied to every
/// object. For the smallest-per-object search instead, see
/// [`to_voxjz_bytes`](crate::to_voxjz_bytes).
pub fn to_voxjz_bytes_with(
    state: &VoxMain,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<Vec<u8>> {
    let file = VoxjFileBuilder::new(state)
        .encoding(Some((position, sample)))
        .build()?;
    Ok(to_voxjz_file_bytes(&file)?)
}
