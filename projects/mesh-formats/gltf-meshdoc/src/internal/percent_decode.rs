/// `uri` with its percent-encoded bytes decoded, the file name it points
/// to, or `None` when a `%` is not followed by two hex digits or the result
/// is not UTF-8.
pub fn percent_decode(uri: &str) -> Option<String> {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = uri.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use crate::percent_decode;

    #[test]
    fn decodes_escapes_and_rejects_a_short_one() {
        assert_eq!(percent_decode("a%20b.png").as_deref(), Some("a b.png"));
        assert_eq!(percent_decode("plain.png").as_deref(), Some("plain.png"));
        assert_eq!(percent_decode("bad%2"), None);
    }
}
