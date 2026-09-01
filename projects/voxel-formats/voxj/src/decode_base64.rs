/// Decodes standard base64 with padding, the text form of the `*-base64`
/// blocks.
pub trait DecodeBase64 {
    /// The bytes `text` encodes, or the reason it is not base64. The block
    /// decoder prefixes the reason with the block it was reading.
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String>;
}
