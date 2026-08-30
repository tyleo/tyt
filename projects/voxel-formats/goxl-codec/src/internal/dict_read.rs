use crate::{ByteReader, Error, Result, invalid};

/// Reads a chunk's trailing `DICT` from `reader`, consuming it to the end.
///
/// Each entry is a `u32` key length, the UTF-8 key bytes, a `u32` value length,
/// then the value bytes; values are kept as raw bytes since the format stores
/// each as a typed binary blob. Reading stops at the end of `reader` or, as
/// Goxel's reader does, at a zero-length key marker. Goxel never writes that
/// marker, so a dictionary normally runs exactly to the chunk's end.
pub fn read_dict(reader: &mut ByteReader) -> Result<Vec<(String, Vec<u8>)>> {
    let mut pairs = Vec::new();
    while !reader.is_empty() {
        let key_len = reader.read_u32()? as usize;
        if key_len == 0 {
            break;
        }
        let key = reader.read_bytes(key_len)?;
        let key = String::from_utf8(key.to_vec())
            .map_err(|error| invalid(format!("dict key is not valid UTF-8: {error}")))?;
        let value_len = reader.read_u32()? as usize;
        let value = reader.read_bytes(value_len)?.to_vec();
        pairs.push((key, value));
    }
    Ok(pairs)
}

/// Removes every pair whose key is `key`, returning the last such value. Taking
/// all occurrences keeps a modeled key from lingering in the leftover `extra`
/// dictionary when a malformed chunk repeats it; keeping the last mirrors Goxel's
/// reader, which copies each matching entry in turn so the final one wins.
pub fn take(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Option<Vec<u8>> {
    let mut value = None;
    pairs.retain(|(pair_key, pair_value)| {
        if pair_key != key {
            return true;
        }
        value = Some(pair_value.clone());
        false
    });
    value
}

/// [`take`], then read the value as a UTF-8 string.
pub fn take_string(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<String>> {
    take(pairs, key)
        .map(|value| as_string(key, &value))
        .transpose()
}

/// [`take`], then read the value as a little-endian `f32`.
pub fn take_f32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<f32>> {
    take(pairs, key)
        .map(|value| as_f32(key, &value))
        .transpose()
}

/// [`take`], then read the value as a little-endian `i32`.
pub fn take_i32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<i32>> {
    take(pairs, key)
        .map(|value| as_i32(key, &value))
        .transpose()
}

/// [`take`], then read the value as Goxel's one-byte boolean.
pub fn take_bool(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<bool>> {
    take(pairs, key)
        .map(|value| as_bool(key, &value))
        .transpose()
}

/// [`take`], then read the value as a `4 x 4` matrix of little-endian `f32`s.
pub fn take_mat4(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[[f32; 4]; 4]>> {
    take(pairs, key)
        .map(|value| as_mat4(key, &value))
        .transpose()
}

/// [`take`], then read the value as four little-endian `f32`s.
pub fn take_vec4f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[f32; 4]>> {
    take(pairs, key)
        .map(|value| as_vec4f(key, &value))
        .transpose()
}

/// [`take`], then read the value as three little-endian `f32`s.
pub fn take_vec3f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[f32; 3]>> {
    take(pairs, key)
        .map(|value| as_vec3f(key, &value))
        .transpose()
}

/// [`take`], then read the value as four `[r, g, b, a]` bytes.
pub fn take_color(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) -> Result<Option<[u8; 4]>> {
    take(pairs, key)
        .map(|value| as_color(key, &value))
        .transpose()
}

/// Reads a value as a UTF-8 string.
fn as_string(key: &str, bytes: &[u8]) -> Result<String> {
    String::from_utf8(bytes.to_vec()).map_err(|error| {
        invalid(format!(
            "dict key {key:?} value is not valid UTF-8: {error}"
        ))
    })
}

/// Reads a value as a little-endian `f32`, requiring exactly four bytes.
fn as_f32(key: &str, bytes: &[u8]) -> Result<f32> {
    Ok(f32::from_le_bytes(fixed(key, bytes, "an f32")?))
}

/// Reads a value as a little-endian `i32`, requiring exactly four bytes.
fn as_i32(key: &str, bytes: &[u8]) -> Result<i32> {
    Ok(i32::from_le_bytes(fixed(key, bytes, "an i32")?))
}

/// Reads a value as Goxel's one-byte boolean, requiring exactly one byte; any
/// non-zero byte is `true`.
fn as_bool(key: &str, bytes: &[u8]) -> Result<bool> {
    Ok(fixed::<1>(key, bytes, "a bool")?[0] != 0)
}

/// Reads a value as a `4 x 4` matrix of little-endian `f32`s in row-major
/// (C array) order, requiring exactly 64 bytes.
fn as_mat4(key: &str, bytes: &[u8]) -> Result<[[f32; 4]; 4]> {
    let bytes = fixed::<64>(key, bytes, "a 4x4 matrix")?;
    let mut matrix = [[0.0f32; 4]; 4];
    for (index, cell) in matrix.iter_mut().flatten().enumerate() {
        let offset = index * 4;
        *cell = f32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
    }
    Ok(matrix)
}

/// Reads a value as four little-endian `f32`s, requiring exactly 16 bytes.
fn as_vec4f(key: &str, bytes: &[u8]) -> Result<[f32; 4]> {
    let bytes = fixed::<16>(key, bytes, "a 4-float vector")?;
    Ok([
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        f32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ])
}

/// Reads a value as three little-endian `f32`s, requiring exactly 12 bytes.
fn as_vec3f(key: &str, bytes: &[u8]) -> Result<[f32; 3]> {
    let bytes = fixed::<12>(key, bytes, "a 3-float vector")?;
    Ok([
        f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        f32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        f32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
    ])
}

/// Reads a value as four `[r, g, b, a]` bytes, requiring exactly four bytes.
fn as_color(key: &str, bytes: &[u8]) -> Result<[u8; 4]> {
    fixed(key, bytes, "an RGBA color")
}

/// Interprets `bytes` as exactly `N` bytes, mapping a length mismatch to an
/// error that names the key and the expected shape.
fn fixed<const N: usize>(key: &str, bytes: &[u8], expected: &str) -> Result<[u8; N]> {
    bytes.try_into().map_err(|_| {
        Error::Invalid(format!(
            "dict key {key:?} expected {N} bytes for {expected}, found {}",
            bytes.len()
        ))
    })
}
