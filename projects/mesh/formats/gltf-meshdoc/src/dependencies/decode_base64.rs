/// Decodes standard base64 with padding, the text form of a data URI.
pub trait DecodeBase64 {
    /// The bytes `text` encodes, or the reason it is not base64.
    fn decode_base64(&self, text: &str) -> Result<Vec<u8>, String>;
}
