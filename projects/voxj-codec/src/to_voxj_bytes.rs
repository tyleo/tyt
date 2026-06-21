use voxj::VoxjFile;

/// Serializes `file` to compact `.voxj` JSON bytes with a trailing newline.
pub fn to_voxj_bytes(file: &VoxjFile) -> serde_json::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(file)?;
    bytes.push(b'\n');
    Ok(bytes)
}
