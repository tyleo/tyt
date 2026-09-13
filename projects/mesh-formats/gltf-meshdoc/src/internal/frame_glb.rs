use crate::{Error, Result};

/// The GLB `JSON` chunk type.
const JSON_CHUNK: u32 = 0x4E4F_534A;

/// The GLB `BIN` chunk type.
const BIN_CHUNK: u32 = 0x004E_4942;

/// Frames `json` and `blob` as a binary glTF. The spec pads the JSON chunk
/// with spaces and the BIN chunk with zeros. No `blob` writes no BIN chunk.
/// Errors if the file would exceed the 4 GiB a GLB can address.
pub fn frame_glb(json: &[u8], blob: Option<&[u8]>) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"glTF");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    // The total length lands here once the chunks are in.
    bytes.extend_from_slice(&0u32.to_le_bytes());

    push_chunk(&mut bytes, JSON_CHUNK, json, b' ')?;

    if let Some(blob) = blob.filter(|blob| !blob.is_empty()) {
        push_chunk(&mut bytes, BIN_CHUNK, blob, 0)?;
    }

    let length = glb_length(bytes.len())?;
    bytes[8..12].copy_from_slice(&length.to_le_bytes());

    Ok(bytes)
}

/// Appends one chunk. The length it records includes the padding.
fn push_chunk(bytes: &mut Vec<u8>, kind: u32, data: &[u8], pad: u8) -> Result<()> {
    let padded = data.len().div_ceil(4) * 4;
    bytes.extend_from_slice(&glb_length(padded)?.to_le_bytes());
    bytes.extend_from_slice(&kind.to_le_bytes());
    bytes.extend_from_slice(data);
    bytes.resize(bytes.len() + (padded - data.len()), pad);
    Ok(())
}

/// `len` as the `u32` a GLB header or chunk records.
fn glb_length(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| Error::invalid("a GLB holds at most 4 GiB"))
}

#[cfg(test)]
mod tests {
    use crate::frame_glb;

    #[test]
    fn frames_and_pads_both_chunks() {
        let bytes = frame_glb(b"{}", Some(&[1, 2, 3, 4, 5])).unwrap();

        assert_eq!(&bytes[0..4], b"glTF");
        assert_eq!(bytes[4..8], 2u32.to_le_bytes());
        assert_eq!(bytes[8..12], (bytes.len() as u32).to_le_bytes());
        assert_eq!(bytes[12..16], 4u32.to_le_bytes());
        assert_eq!(&bytes[16..20], b"JSON");
        assert_eq!(&bytes[20..24], b"{}  ");
        assert_eq!(bytes[24..28], 8u32.to_le_bytes());
        assert_eq!(&bytes[28..32], b"BIN\0");
        assert_eq!(&bytes[32..40], &[1, 2, 3, 4, 5, 0, 0, 0]);
        assert_eq!(bytes.len(), 40);
    }

    #[test]
    fn no_blob_writes_no_bin_chunk() {
        let bytes = frame_glb(b"{}", None).unwrap();

        assert_eq!(bytes.len(), 24);
    }
}
