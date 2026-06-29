use crate::{Result, to_voxj_file_with_encoding};
use voxcore::VoxMain;
use voxj::VoxjFile;
use voxj_codec::{PositionEncoding, SampleEncoding};

/// Encodes a [`VoxMain`] into a [`VoxjFile`] with fixed `position` and `sample`
/// block encodings applied to every object. For the smallest-per-object search
/// instead, see [`to_voxj_file`](crate::to_voxj_file).
pub fn to_voxj_file_with(
    state: &VoxMain,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<VoxjFile> {
    to_voxj_file_with_encoding(state, Some((position, sample)))
}
