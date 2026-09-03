use voxj::VoxjFile;

/// Parses the JSON text of a `.voxj` document into a [`VoxjFile`]. Floats
/// must parse exactly, so a written document reloads to the same values.
pub trait DecodeVoxjJson {
    /// The document `bytes` hold, or the reason they are not one.
    fn decode_voxj_json(&self, bytes: &[u8]) -> Result<VoxjFile, String>;
}
