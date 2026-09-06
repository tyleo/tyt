/// Decompresses an LZFSE block stream, the outer framing of the
/// `contents*.vmaxb`, `*.vmaxhb`, and `*.vmaxhvsb` payloads.
pub trait DecompressLzfse {
    /// The bytes `stream` encodes, or the reason it cannot be decoded. The
    /// caller has already matched the stream's magic, so `stream` starts with
    /// an LZFSE block header.
    fn decompress_lzfse(&self, stream: &[u8]) -> Result<Vec<u8>, String>;
}
