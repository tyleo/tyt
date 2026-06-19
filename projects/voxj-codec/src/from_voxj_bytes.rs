use serde_json::Value;
use std::io;
use voxj::VoxjFile;

/// Decodes `.voxj` JSON bytes into the document and its opaque `main.ext`
/// namespace (JSON null when absent). The codec ascribes no meaning to `ext`;
/// tools read their own format-specific data from under named keys.
pub fn from_voxj_bytes(bytes: &[u8]) -> io::Result<(VoxjFile, Value)> {
    let invalid = |error| io::Error::new(io::ErrorKind::InvalidData, error);
    let mut value: Value = serde_json::from_slice(bytes).map_err(invalid)?;
    let ext = value
        .get_mut("main")
        .and_then(Value::as_object_mut)
        .and_then(|main| main.remove("ext"))
        .unwrap_or(Value::Null);
    let file = serde_json::from_value(value).map_err(invalid)?;
    Ok((file, ext))
}

#[cfg(test)]
mod tests {
    use crate::{VoxjEncoder, from_voxj_bytes, from_voxj_or_voxjz_bytes, from_voxjz_bytes};
    use serde_json::json;
    use voxj::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPositionBlock, VoxjSampleBlock,
        VoxjTransform,
    };

    fn document() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                objects: vec![VoxjObject {
                    name: "o".to_owned(),
                    palette_refs: vec![0],
                    bounds: [2, 1, 1],
                    voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                    voxel_samples: VoxjSampleBlock::RawJson(vec![vec![1], vec![2]]),
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
            },
        }
    }

    #[test]
    fn voxj_round_trips_document_and_ext() {
        let file = document();
        let ext = json!({ "voxel-max": { "scene": { "v": 4 } } });
        let bytes = VoxjEncoder::new(&file)
            .ext(&serde_json::to_vec(&ext).unwrap())
            .json()
            .unwrap();
        let (decoded, decoded_ext) = from_voxj_bytes(&bytes).unwrap();
        assert_eq!(decoded, file);
        assert_eq!(decoded_ext, ext);
    }

    #[test]
    fn voxj_without_ext_decodes_to_null() {
        let file = document();
        let bytes = VoxjEncoder::new(&file).json().unwrap();
        let (decoded, ext) = from_voxj_bytes(&bytes).unwrap();
        assert_eq!(decoded, file);
        assert!(ext.is_null());
    }

    #[test]
    fn voxjz_round_trips_and_detection_dispatches() {
        let file = document();
        let ext = json!({ "k": 1 });
        let zip = VoxjEncoder::new(&file)
            .ext(&serde_json::to_vec(&ext).unwrap())
            .zip()
            .unwrap();
        let (decoded, decoded_ext) = from_voxjz_bytes(&zip).unwrap();
        assert_eq!(decoded, file);
        assert_eq!(decoded_ext, ext);

        // Detection: leading PK -> voxjz, leading { -> voxj.
        assert_eq!(from_voxj_or_voxjz_bytes(&zip).unwrap().0, file);
        let json = VoxjEncoder::new(&file).json().unwrap();
        assert_eq!(from_voxj_or_voxjz_bytes(&json).unwrap().0, file);
    }
}
