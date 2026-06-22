use voxj::VoxjSerdeFile;

/// Serializes `file` to pretty-printed `.voxj` JSON bytes with a trailing
/// newline.
pub fn to_voxj_pretty_bytes(file: &VoxjSerdeFile) -> serde_json::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(file)?;
    bytes.push(b'\n');
    Ok(bytes)
}
