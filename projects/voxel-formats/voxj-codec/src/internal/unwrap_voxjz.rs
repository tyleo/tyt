use crate::{Error, Inflate, Result};

/// Inflates the single deflate member of a `.voxjz` archive back to its `.voxj`
/// bytes through `dependencies`, the inverse of
/// [`wrap_voxjz`](crate::wrap_voxjz). The deflate stream ends itself, so any
/// trailing zip central-directory bytes are ignored.
pub fn unwrap_voxjz<D: Inflate>(dependencies: &D, bytes: &[u8]) -> Result<Vec<u8>> {
    let invalid = || Error::Invalid("malformed .voxjz archive".to_owned());
    if bytes.len() < 30 || bytes[0..4] != [0x50, 0x4b, 0x03, 0x04] {
        return Err(invalid());
    }
    let name_len = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
    let extra_len = u16::from_le_bytes([bytes[28], bytes[29]]) as usize;
    let compressed = bytes.get(30 + name_len + extra_len..).ok_or_else(invalid)?;
    dependencies
        .inflate(compressed)
        .map_err(|reason| Error::Invalid(format!("malformed .voxjz deflate stream: {reason}")))
}
