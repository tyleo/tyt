use voxj::VoxjSerdeFile;

/// Serializes `file` to compact `.voxj` JSON bytes with a trailing newline.
pub fn to_voxj_file_bytes(file: &VoxjSerdeFile) -> serde_json::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec(file)?;
    bytes.push(b'\n');
    Ok(bytes)
}
