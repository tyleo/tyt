/// Decompresses a raw deflate stream, the method of the `.voxjz` archive's
/// member.
pub trait Inflate {
    /// The bytes `stream` encodes, or the reason it is malformed. `stream`
    /// runs past the end of the deflate data into the archive's trailing
    /// records, which the decoder must leave unread.
    fn inflate(&self, stream: &[u8]) -> Result<Vec<u8>, String>;
}
