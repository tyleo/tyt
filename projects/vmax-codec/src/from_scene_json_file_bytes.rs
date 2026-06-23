use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxSceneJsonFile;

/// Decodes `scene.json` bytes (JSON) into a [`VMaxSceneJsonFile`].
pub fn from_scene_json_file_bytes(bytes: &[u8]) -> Result<VMaxSceneJsonFile> {
    serde_json::from_slice(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
