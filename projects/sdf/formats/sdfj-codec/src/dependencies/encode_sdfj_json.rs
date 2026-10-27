use sdfj::SdfjFile;

/// Serializes an [`SdfjFile`] to the JSON text of a `.sdfj` document. A file
/// holding a NaN, an infinity, or a repeated object key has no JSON form.
pub trait EncodeSdfjJson {
    /// The compact JSON of `file`, or the reason `file` has none.
    fn encode_sdfj_json(&self, file: &SdfjFile) -> Result<Vec<u8>, String>;

    /// The pretty-printed JSON of `file`, or the reason `file` has none.
    fn encode_sdfj_json_pretty(&self, file: &SdfjFile) -> Result<Vec<u8>, String>;
}
