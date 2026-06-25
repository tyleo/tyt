use serde::Serialize;
use std::io::{Error, ErrorKind, Result};

/// Serializes `value` to a binary property list — the inverse of
/// [`parse_bplist`](crate::parse_bplist).
pub fn serialize_bplist<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    plist::to_writer_binary(&mut out, value).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::{parse_bplist, serialize_bplist};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Sample {
        name: String,
        values: Vec<u32>,
        flag: bool,
    }

    #[test]
    fn round_trips_through_parse_bplist() {
        let value = Sample {
            name: "palette".to_owned(),
            values: vec![1, 2, 300],
            flag: true,
        };
        let bytes = serialize_bplist(&value).unwrap();
        let decoded: Sample = parse_bplist(&bytes).unwrap();
        assert_eq!(decoded, value);
    }
}
