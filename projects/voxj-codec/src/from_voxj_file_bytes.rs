use crate::{Error, Result};
use voxj::VoxjSerdeFile;

/// Decodes `.voxj` JSON bytes into a [`VoxjSerdeFile`]. Any `main.ext` extension
/// namespace is carried on [`VoxjMain::ext`](voxj::VoxjMain::ext).
pub fn from_voxj_file_bytes(bytes: &[u8]) -> Result<VoxjSerdeFile> {
    serde_json::from_slice(bytes).map_err(Error::Json)
}

#[cfg(test)]
mod tests {
    use crate::{
        from_voxj_file_bytes, from_voxj_or_voxjz_file_bytes, from_voxjz_file_bytes,
        to_voxj_file_bytes, to_voxjz_file_bytes,
    };
    use serde_json::{Value, json};
    use voxj::{
        VoxjHierarchyNode, VoxjMain, VoxjSerdeFile, VoxjSerdeObject, VoxjSerdePositionBlock,
        VoxjSerdeSampleBlock, VoxjTransform, VoxjValue,
    };

    fn document() -> VoxjSerdeFile {
        VoxjSerdeFile {
            version: 1,
            main: VoxjMain {
                objects: vec![VoxjSerdeObject {
                    name: "o".to_owned(),
                    palette_refs: vec![0],
                    bounds: [2, 1, 1],
                    voxel_positions: VoxjSerdePositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                    voxel_samples: VoxjSerdeSampleBlock::RawJson(vec![vec![1], vec![2]]),
                }],
                palettes: Vec::new(),
                hierarchy_nodes: vec![VoxjHierarchyNode {
                    name: "o".to_owned(),
                    child_nodes: Vec::new(),
                    child_objects: vec![0],
                    transform: VoxjTransform {
                        position: [0.0; 3],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0; 3],
                    },
                }],
                root_hierarchy_nodes: vec![0],
                ext: None,
            },
        }
    }

    fn document_with_ext(ext: Value) -> VoxjSerdeFile {
        let mut file = document();
        file.main.ext = Some(serde_json::from_value::<VoxjValue>(ext).unwrap());
        file
    }

    #[test]
    fn voxj_round_trips_document_and_ext() {
        let file = document_with_ext(json!({ "voxel-max": { "scene": { "v": 4 } } }));
        let bytes = to_voxj_file_bytes(&file).unwrap();
        assert_eq!(from_voxj_file_bytes(&bytes).unwrap(), file);
    }

    #[test]
    fn voxj_without_ext_omits_the_field() {
        let file = document();
        let bytes = to_voxj_file_bytes(&file).unwrap();
        assert!(
            !String::from_utf8(bytes.clone())
                .unwrap()
                .contains("\"ext\"")
        );
        let decoded = from_voxj_file_bytes(&bytes).unwrap();
        assert_eq!(decoded, file);
        assert!(decoded.main.ext.is_none());
    }

    #[test]
    fn voxjz_round_trips_and_detection_dispatches() {
        let file = document_with_ext(json!({ "k": 1 }));
        let zip = to_voxjz_file_bytes(&file).unwrap();
        assert_eq!(from_voxjz_file_bytes(&zip).unwrap(), file);

        // Detection: leading PK -> voxjz, leading { -> voxj.
        assert_eq!(from_voxj_or_voxjz_file_bytes(&zip).unwrap(), file);
        let json = to_voxj_file_bytes(&file).unwrap();
        assert_eq!(from_voxj_or_voxjz_file_bytes(&json).unwrap(), file);
    }
}
