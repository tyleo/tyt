use sdfj::SdfjFile;

/// Parses the JSON text of a `.sdfj` document into an [`SdfjFile`]. Floats
/// must parse exactly, so a written document reloads to the same values.
pub trait DecodeSdfjJson {
    /// The document `bytes` hold, or the reason they are not one.
    fn decode_sdfj_json(&self, bytes: &[u8]) -> Result<SdfjFile, String>;
}
