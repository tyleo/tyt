use crate::VoxjFileSerde;
use voxj::VoxjFile;

/// Serializes a document to `.voxj` bytes: a single UTF-8 JSON value, compact
/// by default or pretty-printed for readability.
pub fn to_voxj_bytes(file: &VoxjFile, pretty: bool) -> serde_json::Result<Vec<u8>> {
    let serde = VoxjFileSerde::from(file.clone());
    let mut bytes = if pretty {
        serde_json::to_vec_pretty(&serde)?
    } else {
        serde_json::to_vec(&serde)?
    };
    bytes.push(b'\n');
    Ok(bytes)
}
