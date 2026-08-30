use crate::{Error, Result};

/// Decodes an unsigned-LEB128 varint byte stream back into integers. Rejects a
/// stream that ends mid-value (a final byte with its continuation bit set) or a
/// value that does not fit in 64 bits, so a malformed hilbert block errors
/// rather than panicking or wrapping.
pub fn decode_varint(bytes: &[u8]) -> Result<Vec<u64>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let mut v = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = *bytes
                .get(i)
                .ok_or_else(|| Error::Invalid("varint stream ends mid-value".to_owned()))?;
            i += 1;
            if shift >= 64 {
                return Err(Error::Invalid("varint value overflows 64 bits".to_owned()));
            }
            v |= ((byte & 0x7f) as u64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        out.push(v);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::{decode_varint, encode_varint};

    #[test]
    fn round_trips_multibyte() {
        let values = [0, 1, 127, 128, 300, 16384, 1_000_000, u32::MAX as u64];
        assert_eq!(decode_varint(&encode_varint(&values)).unwrap(), values);
    }

    /// A final byte with its continuation bit set has no successor, so the
    /// stream ends mid-value and must error rather than panic.
    #[test]
    fn rejects_truncated_stream() {
        assert!(decode_varint(&[0x80]).is_err());
        assert!(decode_varint(&[0x00, 0x81]).is_err());
    }

    /// Ten continuation bytes then a terminator encode a 70-bit value, which
    /// overflows 64 bits and is rejected rather than silently truncated.
    #[test]
    fn rejects_overlong_value() {
        let mut bytes = vec![0x80; 10];
        bytes.push(0x00);
        assert!(decode_varint(&bytes).is_err());
    }
}
