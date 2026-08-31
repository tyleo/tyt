use crate::{Result, VoxjFileBuilder, VoxjVoxMain};
use voxj::objects::{PositionEncoding, SampleEncoding};
use voxj_codec::to_voxj_file_bytes;

/// Writes a [`VoxjVoxMain`] to compact `.voxj` JSON bytes with fixed
/// `position` and `sample` block encodings applied to every object. For the
/// smallest-per-object search instead, see
/// [`to_voxj_bytes`](crate::codec::to_voxj_bytes).
pub fn to_voxj_bytes_with(
    state: &VoxjVoxMain,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<Vec<u8>> {
    let file = VoxjFileBuilder::new(state)
        .position_encoding(Some(position))
        .sample_encoding(Some(sample))
        .build()?;
    Ok(to_voxj_file_bytes(&file)?)
}
