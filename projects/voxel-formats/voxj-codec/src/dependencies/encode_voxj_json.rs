use voxj::VoxjFile;

/// Serializes a [`VoxjFile`] to the JSON text of a `.voxj` document.
pub trait EncodeVoxjJson {
    /// The compact JSON of `file`.
    fn encode_voxj_json(&self, file: &VoxjFile) -> Vec<u8>;

    /// The pretty-printed JSON of `file`.
    fn encode_voxj_json_pretty(&self, file: &VoxjFile) -> Vec<u8>;
}
