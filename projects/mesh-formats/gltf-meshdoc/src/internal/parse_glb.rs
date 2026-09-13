use crate::{Error, Result};

/// The GLB `JSON` chunk type.
const JSON_CHUNK: u32 = 0x4E4F_534A;

/// The GLB `BIN` chunk type.
const BIN_CHUNK: u32 = 0x004E_4942;

/// Whether `bytes` start with the GLB magic.
pub fn is_glb(bytes: &[u8]) -> bool {
    bytes.starts_with(b"glTF")
}

/// The JSON chunk and the optional BIN chunk of a binary glTF. Errors on a
/// header or chunk that does not fit the container spec.
pub fn parse_glb(bytes: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
    if bytes.len() < 12 || !is_glb(bytes) {
        return Err(Error::invalid("not a GLB: the header is missing its magic"));
    }

    let version = u32_at(bytes, 4);
    if version != 2 {
        return Err(Error::invalid(format!("GLB version {version} is not 2")));
    }

    let length = u32_at(bytes, 8) as usize;
    if length > bytes.len() {
        return Err(Error::invalid(format!(
            "GLB header declares {length} bytes but {} were given",
            bytes.len()
        )));
    }

    let mut json = None;
    let mut bin = None;
    let mut cursor = 12;

    while cursor + 8 <= length {
        let chunk_length = u32_at(bytes, cursor) as usize;
        let chunk_type = u32_at(bytes, cursor + 4);
        let start = cursor + 8;
        let end = start
            .checked_add(chunk_length)
            .filter(|&end| end <= length)
            .ok_or_else(|| Error::invalid("a GLB chunk runs past the file"))?;

        let data = &bytes[start..end];

        match chunk_type {
            JSON_CHUNK if json.is_none() => json = Some(data.to_vec()),
            BIN_CHUNK if bin.is_none() => bin = Some(data.to_vec()),
            JSON_CHUNK | BIN_CHUNK => {
                return Err(Error::invalid("a GLB repeats a JSON or BIN chunk"));
            }
            _ => {}
        }

        cursor = end;
    }

    let json = json.ok_or_else(|| Error::invalid("a GLB has no JSON chunk"))?;

    Ok((json, bin))
}

/// The little-endian `u32` at `offset`.
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("a four-byte slice"),
    )
}

#[cfg(test)]
mod tests {
    use crate::{frame_glb, parse_glb};

    #[test]
    fn parses_what_the_framer_writes() {
        let bytes = frame_glb(b"{}", Some(&[1, 2, 3, 4, 5])).unwrap();

        let (json, bin) = parse_glb(&bytes).unwrap();

        // The JSON chunk keeps its space padding; the BIN chunk its zeros.
        assert_eq!(json, b"{}  ");
        assert_eq!(bin, Some(vec![1, 2, 3, 4, 5, 0, 0, 0]));

        let (_, bin) = parse_glb(&frame_glb(b"{}", None).unwrap()).unwrap();
        assert_eq!(bin, None);
    }

    #[test]
    fn rejects_a_bad_header() {
        assert!(parse_glb(b"glTF").is_err());
        assert!(parse_glb(b"not a glb at all").is_err());
    }
}
