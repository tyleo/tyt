use crate::EncodeVoxjJson;
use voxj::VoxjFile;

/// Serializes `file` through `dependencies` to pretty-printed `.voxj` JSON
/// bytes with a trailing newline.
pub fn to_voxj_pretty_file_bytes<D: EncodeVoxjJson>(dependencies: &D, file: &VoxjFile) -> Vec<u8> {
    let mut bytes = dependencies.encode_voxj_json_pretty(file);
    bytes.push(b'\n');
    bytes
}
