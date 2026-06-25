/// Pushes `key` with `value`'s shortest round-tripping decimal form, when set.
pub fn push_f32(pairs: &mut Vec<(String, String)>, key: &str, value: Option<f32>) {
    if let Some(value) = value {
        pairs.push((key.to_owned(), value.to_string()));
    }
}

/// Pushes `key` with `value`'s decimal form, when set.
pub fn push_i32(pairs: &mut Vec<(String, String)>, key: &str, value: Option<i32>) {
    if let Some(value) = value {
        pairs.push((key.to_owned(), value.to_string()));
    }
}

/// Pushes `key` with `value` as a space-separated triple, when set.
pub fn push_vec3f(pairs: &mut Vec<(String, String)>, key: &str, value: Option<[f32; 3]>) {
    if let Some([a, b, c]) = value {
        pairs.push((key.to_owned(), format!("{a} {b} {c}")));
    }
}

/// Pushes `key` with `value` as a space-separated triple.
pub fn push_vec3i(pairs: &mut Vec<(String, String)>, key: &str, value: [i32; 3]) {
    let [a, b, c] = value;
    pairs.push((key.to_owned(), format!("{a} {b} {c}")));
}

/// Pushes `key` with a string value, when set.
pub fn push_str(pairs: &mut Vec<(String, String)>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        pairs.push((key.to_owned(), value.to_owned()));
    }
}

/// Pushes `key` with MagicaVoxel's `0` / `1` flag encoding, when set.
pub fn push_bool(pairs: &mut Vec<(String, String)>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        pairs.push((key.to_owned(), if value { "1" } else { "0" }.to_owned()));
    }
}
