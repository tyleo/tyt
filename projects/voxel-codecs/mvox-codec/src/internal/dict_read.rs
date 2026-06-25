use crate::{Error, Result};

/// An [`Error::Invalid`] error carrying `message`, for input that is
/// well-framed but semantically malformed.
pub fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

/// Removes every pair whose key is `key`, returning the first such value. Taking
/// all occurrences (not just the first) keeps a modeled key from lingering in
/// the leftover `extra` dictionary when a malformed `DICT` repeats it, which
/// would otherwise re-lift onto the typed field on the next decode.
pub fn take(pairs: &mut Vec<(String, String)>, key: &str) -> Option<String> {
    let mut value = None;
    pairs.retain(|(pair_key, pair_value)| {
        if pair_key != key {
            return true;
        }
        if value.is_none() {
            value = Some(pair_value.clone());
        }
        false
    });
    value
}

/// [`take`], then parse the value as an `f32`.
pub fn take_f32(pairs: &mut Vec<(String, String)>, key: &str) -> Result<Option<f32>> {
    take(pairs, key).map(|value| parse_f32(&value)).transpose()
}

/// [`take`], then parse the value as an `i32`.
pub fn take_i32(pairs: &mut Vec<(String, String)>, key: &str) -> Result<Option<i32>> {
    take(pairs, key).map(|value| parse_i32(&value)).transpose()
}

/// [`take`], then parse the value as a `u32`.
pub fn take_u32(pairs: &mut Vec<(String, String)>, key: &str) -> Result<Option<u32>> {
    take(pairs, key).map(|value| parse_u32(&value)).transpose()
}

/// [`take`], then read the value as a `0` / `1` boolean flag.
pub fn take_bool(pairs: &mut Vec<(String, String)>, key: &str) -> Option<bool> {
    take(pairs, key).map(|value| parse_bool(&value))
}

/// [`take`], then parse the value as a space-separated `[f32; 3]`.
pub fn take_vec3f(pairs: &mut Vec<(String, String)>, key: &str) -> Result<Option<[f32; 3]>> {
    take(pairs, key)
        .map(|value| parse_vec3f(&value))
        .transpose()
}

/// [`take`], then parse the value as a space-separated `[i32; 3]`.
pub fn take_vec3i(pairs: &mut Vec<(String, String)>, key: &str) -> Result<Option<[i32; 3]>> {
    take(pairs, key)
        .map(|value| parse_vec3i(&value))
        .transpose()
}

/// Parses an `f32`, mapping a malformed value to an error.
pub fn parse_f32(value: &str) -> Result<f32> {
    value
        .trim()
        .parse()
        .map_err(|_| invalid(format!("expected a float, found {value:?}")))
}

/// Parses an `i32`, mapping a malformed value to an error.
pub fn parse_i32(value: &str) -> Result<i32> {
    value
        .trim()
        .parse()
        .map_err(|_| invalid(format!("expected an integer, found {value:?}")))
}

/// Parses a `u32`, mapping a malformed value to an error.
pub fn parse_u32(value: &str) -> Result<u32> {
    value
        .trim()
        .parse()
        .map_err(|_| invalid(format!("expected an unsigned integer, found {value:?}")))
}

/// Reads MagicaVoxel's `0` / `1` flag encoding; any value other than `"1"`
/// (including a missing key, handled by the caller) reads as `false`.
pub fn parse_bool(value: &str) -> bool {
    value.trim() == "1"
}

/// Parses three space-separated floats.
pub fn parse_vec3f(value: &str) -> Result<[f32; 3]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let [a, b, c] = parts[..] else {
        return Err(invalid(format!("expected three floats, found {value:?}")));
    };
    Ok([parse_f32(a)?, parse_f32(b)?, parse_f32(c)?])
}

/// Parses three space-separated integers.
pub fn parse_vec3i(value: &str) -> Result<[i32; 3]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    let [a, b, c] = parts[..] else {
        return Err(invalid(format!("expected three integers, found {value:?}")));
    };
    Ok([parse_i32(a)?, parse_i32(b)?, parse_i32(c)?])
}

#[cfg(test)]
mod tests {
    use crate::take;

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    #[test]
    fn take_removes_every_occurrence_and_returns_the_first() {
        // A repeated modeled key must not survive in the leftover `extra`.
        let mut pairs = vec![pair("_r", "4"), pair("keep", "x"), pair("_r", "105")];
        assert_eq!(take(&mut pairs, "_r"), Some("4".to_owned()));
        assert_eq!(pairs, vec![pair("keep", "x")]);
        assert_eq!(take(&mut pairs, "_r"), None);
    }
}
