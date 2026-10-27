use voxj::VoxjFile;

/// Serializes a [`VoxjFile`] to the JSON text of a `.voxj` document. A file
/// holding a NaN, an infinity outside a `float` value pool, or a repeated
/// object key has no JSON form.
pub trait EncodeVoxjJson {
    /// The compact JSON of `file`, or the reason `file` has none.
    fn encode_voxj_json(&self, file: &VoxjFile) -> Result<Vec<u8>, String>;

    /// The pretty-printed JSON of `file`, or the reason `file` has none.
    fn encode_voxj_json_pretty(&self, file: &VoxjFile) -> Result<Vec<u8>, String>;
}
