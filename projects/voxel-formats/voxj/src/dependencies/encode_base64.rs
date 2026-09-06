/// Encodes bytes as standard base64 with padding, the text form of the
/// `*-base64` blocks.
pub trait EncodeBase64 {
    /// The base64 text of `bytes`.
    fn encode_base64(&self, bytes: &[u8]) -> String;
}
