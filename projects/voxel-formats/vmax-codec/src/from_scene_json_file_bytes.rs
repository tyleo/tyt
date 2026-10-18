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

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{DependenciesImpl, from_scene_json_file_bytes, to_scene_json_file_bytes};
    use vmax::VMaxDiscarded;

    /// The active object is editor state: it reads, keeps nothing, and is
    /// never written, so a written scene opens on the scene.
    #[test]
    fn reads_and_discards_the_active_object() {
        let scene =
            from_scene_json_file_bytes(&DependenciesImpl, br#"{"objects":[],"ao":0}"#).unwrap();

        assert_eq!(scene.ao, Some(VMaxDiscarded));

        let written = to_scene_json_file_bytes(&DependenciesImpl, &scene).unwrap();

        assert!(!String::from_utf8(written).unwrap().contains("\"ao\""));
    }
}
