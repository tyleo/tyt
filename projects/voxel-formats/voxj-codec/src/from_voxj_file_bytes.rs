use crate::{DecodeVoxjJson, Error, Result};
use voxj::VoxjFile;

/// Decodes `.voxj` JSON bytes into a [`VoxjFile`] through `dependencies`.
pub fn from_voxj_file_bytes<D: DecodeVoxjJson>(dependencies: &D, bytes: &[u8]) -> Result<VoxjFile> {
    dependencies.decode_voxj_json(bytes).map_err(Error::Json)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, from_voxj_file_bytes, from_voxj_or_voxjz_file_bytes,
        from_voxjz_file_bytes, to_voxj_file_bytes, to_voxjz_file_bytes,
    };
    use serde_json::{Value, json};
    use voxj::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjMap, VoxjObject, VoxjPositionBlock,
        VoxjRuntimeState, VoxjSampleBlock, VoxjTransform, VoxjValuePool,
    };

    fn document() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: Vec::new(),
                    palettes: Vec::new(),
                    objects: vec![VoxjObject {
                        name: "o".to_owned(),
                        layers: vec![0],
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1, 2]]),
                    }],
                    nodes: vec![VoxjHierarchyNode {
                        name: "o".to_owned(),
                        child_nodes: Vec::new(),
                        child_objects: vec![0],
                        transform: VoxjTransform {
                            position: [0.0; 3],
                            rotation: [0.0, 0.0, 0.0, 1.0],
                            scale: [1.0; 3],
                        },
                    }],
                    root_nodes: vec![0],
                },
                edit_state: None,
                ext: None,
            },
        }
    }

    fn document_with_ext(ext: Value) -> VoxjFile {
        let mut file = document();
        file.main.ext = Some(serde_json::from_value::<VoxjMap>(ext).unwrap());
        file
    }

    #[test]
    fn voxj_round_trips_document_and_ext() {
        let file = document_with_ext(json!({ "vmax": { "scene": { "v": 4 } } }));
        let bytes = to_voxj_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_voxj_file_bytes(&DependenciesImpl, &bytes).unwrap(),
            file
        );
    }

    #[test]
    fn voxj_without_ext_omits_the_field() {
        let file = document();
        let bytes = to_voxj_file_bytes(&DependenciesImpl, &file);
        assert!(
            !String::from_utf8(bytes.clone())
                .unwrap()
                .contains("\"ext\"")
        );
        let decoded = from_voxj_file_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(decoded, file);
        assert!(decoded.main.ext.is_none());
    }

    // These three values mis-parse by one ULP without serde_json's
    // float_roundtrip feature, so this round trip proves the manifest
    // carries it.
    #[test]
    fn seventeen_digit_floats_save_and_load_byte_identical() {
        let mut file = document();
        file.main.runtime_state.value_pools = vec![VoxjValuePool::Float(vec![
            0.0009105809506465125,
            0.21586050011389926,
            0.9734452903984125,
        ])];
        let bytes = to_voxj_file_bytes(&DependenciesImpl, &file);
        let reloaded = from_voxj_file_bytes(&DependenciesImpl, &bytes).unwrap();
        assert_eq!(to_voxj_file_bytes(&DependenciesImpl, &reloaded), bytes);
    }

    #[test]
    fn voxjz_round_trips_and_detection_dispatches() {
        let file = document_with_ext(json!({ "k": 1 }));
        let zip = to_voxjz_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_voxjz_file_bytes(&DependenciesImpl, &zip).unwrap(),
            file
        );

        // Detection: leading PK -> voxjz, leading { -> voxj.
        assert_eq!(
            from_voxj_or_voxjz_file_bytes(&DependenciesImpl, &zip).unwrap(),
            file
        );
        let json = to_voxj_file_bytes(&DependenciesImpl, &file);
        assert_eq!(
            from_voxj_or_voxjz_file_bytes(&DependenciesImpl, &json).unwrap(),
            file
        );
    }

    #[test]
    fn undecodable_json_reports_the_parse_failure() {
        let error = from_voxj_file_bytes(&DependenciesImpl, b"not a document").unwrap_err();
        assert!(matches!(error, crate::Error::Json(_)));
        assert!(!error.to_string().is_empty());
    }
}
