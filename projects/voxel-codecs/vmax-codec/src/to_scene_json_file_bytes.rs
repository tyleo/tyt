use crate::{Error, Result};
use vmax::VMaxSceneJsonFile;

/// Encodes a [`VMaxSceneJsonFile`] into compact `scene.json` bytes, the
/// inverse of
/// [`from_scene_json_file_bytes`](crate::from_scene_json_file_bytes). Voxel Max
/// writes `scene.json` compact (no indentation), so this serializes compact
/// too.
pub fn to_scene_json_file_bytes(file: &VMaxSceneJsonFile) -> Result<Vec<u8>> {
    serde_json::to_vec(file).map_err(Error::Json)
}
