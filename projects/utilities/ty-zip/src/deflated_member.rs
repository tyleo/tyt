/// The deflate stream of the first member in `archive`, the inverse of
/// [`wrap_deflated_member`](crate::wrap_deflated_member()). Returns `None` when
/// `archive` does not open with a local file header. The stream runs on into
/// the archive's trailing records, so the decoder must stop at the end of the
/// deflate data.
pub fn deflated_member(archive: &[u8]) -> Option<&[u8]> {
    if archive.len() < 30 || archive[0..4] != [0x50, 0x4b, 0x03, 0x04] {
        return None;
    }
    let name_len = u16::from_le_bytes([archive[26], archive[27]]) as usize;
    let extra_len = u16::from_le_bytes([archive[28], archive[29]]) as usize;
    archive.get(30 + name_len + extra_len..)
}

#[cfg(test)]
mod tests {
    use crate::deflated_member;

    #[test]
    fn bytes_without_a_local_header_hold_no_member() {
        assert_eq!(deflated_member(b"{\"version\":1}"), None);
        assert_eq!(deflated_member(b"PK\x03\x04short"), None);
    }
}
