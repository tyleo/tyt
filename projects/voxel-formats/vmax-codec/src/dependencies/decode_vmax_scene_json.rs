use vmax::VMaxSceneJsonFile;

/// Parses the JSON text of `scene.json` into a [`VMaxSceneJsonFile`]. Floats
/// must parse exactly, so a written scene reloads to the same values.
pub trait DecodeVMaxSceneJson {
    /// The scene `bytes` hold, or the reason they are not one.
    fn decode_vmax_scene_json(&self, bytes: &[u8]) -> Result<VMaxSceneJsonFile, String>;
}
