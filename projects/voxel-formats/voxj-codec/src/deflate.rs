/// Compresses bytes into a raw deflate stream, the method of the `.voxjz`
/// archive's member.
pub trait Deflate {
    /// The deflate stream of `bytes`. A deterministic encoder keeps archives
    /// reproducible.
    fn deflate(&self, bytes: &[u8]) -> Vec<u8>;
}
