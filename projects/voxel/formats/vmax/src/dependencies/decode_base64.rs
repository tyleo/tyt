/// Decodes standard base64 with padding, the text a
/// [`VMaxOpaqueFile`](crate::VMaxOpaqueFile) takes outside its package.
pub trait DecodeBase64 {
    /// The bytes `text` encodes, or the reason it is not base64. The caller
    /// prefixes the reason with the file it was reading.
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String>;
}
