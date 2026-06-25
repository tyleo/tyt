/// Pushes `key` with raw `value` bytes.
pub fn push_bytes(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: &[u8]) {
    pairs.push((key.to_owned(), value.to_vec()));
}

/// Pushes `key` with a UTF-8 string value.
pub fn push_str(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: &str) {
    push_bytes(pairs, key, value.as_bytes());
}

/// Pushes `key` with a little-endian `f32` value.
pub fn push_f32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: f32) {
    push_bytes(pairs, key, &value.to_le_bytes());
}

/// Pushes `key` with a little-endian `i32` value.
pub fn push_i32(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: i32) {
    push_bytes(pairs, key, &value.to_le_bytes());
}

/// Pushes `key` with Goxel's one-byte boolean encoding.
pub fn push_bool(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: bool) {
    push_bytes(pairs, key, &[value as u8]);
}

/// Pushes `key` with a `4 x 4` matrix of little-endian `f32`s in row-major
/// (C array) order, the inverse of the matrix read in `dict_read`.
pub fn push_mat4(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: [[f32; 4]; 4]) {
    let mut bytes = Vec::with_capacity(64);
    for cell in value.iter().flatten() {
        bytes.extend_from_slice(&cell.to_le_bytes());
    }
    pairs.push((key.to_owned(), bytes));
}

/// Pushes `key` with four little-endian `f32`s.
pub fn push_vec4f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: [f32; 4]) {
    let mut bytes = Vec::with_capacity(16);
    for component in value {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
    pairs.push((key.to_owned(), bytes));
}

/// Pushes `key` with three little-endian `f32`s.
pub fn push_vec3f(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: [f32; 3]) {
    let mut bytes = Vec::with_capacity(12);
    for component in value {
        bytes.extend_from_slice(&component.to_le_bytes());
    }
    pairs.push((key.to_owned(), bytes));
}

/// Pushes `key` with four `[r, g, b, a]` bytes.
pub fn push_color(pairs: &mut Vec<(String, Vec<u8>)>, key: &str, value: [u8; 4]) {
    push_bytes(pairs, key, &value);
}

/// Pushes `key` as a marker: present, with a zero-length value. Goxel uses this
/// for the active-camera flag.
pub fn push_marker(pairs: &mut Vec<(String, Vec<u8>)>, key: &str) {
    pairs.push((key.to_owned(), Vec::new()));
}
