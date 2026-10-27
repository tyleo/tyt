use crate::{EncodeVoxjJson, Error, Result};
use voxj::VoxjFile;

/// Serializes `file` through `dependencies` to compact `.voxj` JSON bytes
/// with a trailing newline.
pub fn to_voxj_file_bytes<D: EncodeVoxjJson>(dependencies: &D, file: &VoxjFile) -> Result<Vec<u8>> {
    let mut bytes = dependencies.encode_voxj_json(file).map_err(Error::Json)?;
    bytes.push(b'\n');
    Ok(bytes)
}
