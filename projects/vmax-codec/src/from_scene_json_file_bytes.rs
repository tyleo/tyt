use crate::{Error, Result};
use vmax::VMaxSceneJsonFile;

/// Decodes `scene.json` bytes (JSON) into a [`VMaxSceneJsonFile`].
pub fn from_scene_json_file_bytes(bytes: &[u8]) -> Result<VMaxSceneJsonFile> {
    serde_json::from_slice(bytes).map_err(Error::Json)
}
