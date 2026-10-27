use vmax::VMaxSceneJsonFile;

/// Serializes a [`VMaxSceneJsonFile`] to the JSON text of `scene.json`. A scene
/// holding a NaN, an infinity, or a data blob has no JSON form.
pub trait EncodeVMaxSceneJson {
    /// The compact JSON of `file`, the form Voxel Max writes, or the reason
    /// `file` has none.
    fn encode_vmax_scene_json(&self, file: &VMaxSceneJsonFile) -> Result<Vec<u8>, String>;
}
