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

    /// The active object, active group, opening level, and node selection
    /// are editor state: each reads, keeps nothing, and is never written.
    #[test]
    fn reads_and_discards_the_selection_and_opening_level() {
        let bytes = br#"{"groups":[{"id":"g","t_p":[0,0,0],"t_r":[0,0,0,0],"t_s":[1,1,1],"s":true}],"objects":[],"ao":0,"ag":0,"vl":0}"#;

        let scene = from_scene_json_file_bytes(&DependenciesImpl, bytes).unwrap();

        assert_eq!(
            (scene.ao, scene.ag, scene.vl, scene.groups[0].s),
            (
                Some(VMaxDiscarded),
                Some(VMaxDiscarded),
                Some(VMaxDiscarded),
                Some(VMaxDiscarded)
            )
        );

        let written = to_scene_json_file_bytes(&DependenciesImpl, &scene).unwrap();

        let written = String::from_utf8(written).unwrap();

        for key in ["\"ao\"", "\"ag\"", "\"vl\"", "\"s\""] {
            assert!(!written.contains(key), "{key} in {written}");
        }
    }
}
