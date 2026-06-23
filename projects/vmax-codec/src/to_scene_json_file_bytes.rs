use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxSceneJsonFile;

/// Encodes a [`VMaxSceneJsonFile`] into pretty-printed `scene.json` bytes — the
/// inverse of [`from_scene_json_file_bytes`](crate::from_scene_json_file_bytes).
pub fn to_scene_json_file_bytes(file: &VMaxSceneJsonFile) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(file).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
