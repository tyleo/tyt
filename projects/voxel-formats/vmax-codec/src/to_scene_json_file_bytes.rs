use crate::EncodeVMaxSceneJson;
use vmax::VMaxSceneJsonFile;

/// Encodes a [`VMaxSceneJsonFile`] into compact `scene.json` bytes through
/// `dependencies`, the inverse of
/// [`from_scene_json_file_bytes`](crate::from_scene_json_file_bytes). Voxel Max
/// writes `scene.json` compact (no indentation), so this serializes compact
/// too.
pub fn to_scene_json_file_bytes<D: EncodeVMaxSceneJson>(
    dependencies: &D,
    file: &VMaxSceneJsonFile,
) -> Vec<u8> {
    dependencies.encode_vmax_scene_json(file)
}
