/// Compresses bytes into an LZFSE block stream, the outer framing of the
/// `contents*.vmaxb`, `*.vmaxhb`, and `*.vmaxhvsb` payloads.
pub trait CompressLzfse {
    /// The LZFSE stream of `bytes`.
    fn compress_lzfse(&self, bytes: &[u8]) -> Vec<u8>;
}
