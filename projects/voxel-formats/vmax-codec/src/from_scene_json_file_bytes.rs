use crate::{DecodeVMaxSceneJson, Error, Result};
use vmax::VMaxSceneJsonFile;

/// Decodes `scene.json` bytes into a [`VMaxSceneJsonFile`] through
/// `dependencies`.
pub fn from_scene_json_file_bytes<D: DecodeVMaxSceneJson>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxSceneJsonFile> {
    dependencies
        .decode_vmax_scene_json(bytes)
        .map_err(Error::Json)
}
