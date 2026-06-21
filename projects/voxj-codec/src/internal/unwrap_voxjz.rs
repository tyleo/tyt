use flate2::read::DeflateDecoder;
use std::io::{self, Read};

/// Inflates the single deflate member of a `.voxjz` archive back to its `.voxj`
/// bytes: the inverse of [`wrap_voxjz`](crate::wrap_voxjz). The deflate stream
/// ends itself, so any trailing zip central-directory bytes are ignored.
pub(crate) fn unwrap_voxjz(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "malformed .voxjz archive");
    if bytes.len() < 30 || bytes[0..4] != [0x50, 0x4b, 0x03, 0x04] {
        return Err(invalid());
    }
    let name_len = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
    let extra_len = u16::from_le_bytes([bytes[28], bytes[29]]) as usize;
    let compressed = bytes.get(30 + name_len + extra_len..).ok_or_else(invalid)?;
    let mut out = Vec::new();
    DeflateDecoder::new(compressed).read_to_end(&mut out)?;
    Ok(out)
}
