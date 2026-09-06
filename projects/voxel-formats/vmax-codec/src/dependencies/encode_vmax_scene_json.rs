use vmax::VMaxSceneJsonFile;

/// Serializes a [`VMaxSceneJsonFile`] to the JSON text of `scene.json`.
pub trait EncodeVMaxSceneJson {
    /// The compact JSON of `file`, the form Voxel Max writes.
    fn encode_vmax_scene_json(&self, file: &VMaxSceneJsonFile) -> Vec<u8>;
}
