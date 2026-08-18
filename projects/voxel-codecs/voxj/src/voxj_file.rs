use crate::VoxjMain;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of a Voxel Json document.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjFile {
    /// Format version.
    pub version: u32,

    /// The document body.
    pub main: VoxjMain,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{
        VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjMap, VoxjObject, VoxjPalette, VoxjPositionBlock,
        VoxjProperty, VoxjRuntimeState, VoxjSampleBlock, VoxjTransform, VoxjValue, VoxjValuePool,
    };
    use serde_json::{Value, json};

    /// Covers the full palette surface: two palettes sharing a value pool, and
    /// one channel per layer. `ext` holds a null, which is a legal value inside
    /// the namespace even though a null `ext` itself rejects.
    fn document() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        VoxjValuePool::Float(vec![1.0, 2.5]),
                        VoxjValuePool::Vec4Float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]),
                    ],
                    palettes: vec![
                        VoxjPalette {
                            properties: vec![VoxjProperty {
                                name: "baseColor".to_owned(),
                                value_pool: 1,
                            }],
                            materials: vec![vec![0], vec![1]],
                        },
                        VoxjPalette {
                            properties: vec![VoxjProperty {
                                name: "emissiveStrength".to_owned(),
                                value_pool: 0,
                            }],
                            materials: vec![vec![1]],
                        },
                    ],
                    objects: vec![VoxjObject {
                        name: "o".to_owned(),
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]]),
                        layers: vec![0, 1],
                        voxel_samples: VoxjSampleBlock::RawJson(vec![vec![0, 1], vec![0, 0]]),
                    }],
                    nodes: vec![VoxjHierarchyNode {
                        name: "o".to_owned(),
                        transform: VoxjTransform {
                            position: [0.0; 3],
                            rotation: [0.0, 0.0, 0.0, 1.0],
                            scale: [1.0; 3],
                        },
                        child_nodes: Vec::new(),
                        child_objects: vec![0],
                    }],
                    root_nodes: vec![0],
                },
                edit_state: None,
                ext: Some(VoxjMap(vec![("vendor".to_owned(), VoxjValue::Null)])),
            },
        }
    }

    /// The wire form of [`document`], spelling out every renamed field.
    fn wire_document() -> Value {
        json!({
            "version": 1,
            "main": {
                "runtimeState": {
                    "valuePools": [
                        { "kind": "float", "values": [1, 2.5] },
                        {
                            "kind": "vec-4-float",
                            "values": [[1, 0, 0, 1], [0, 1, 0, 1]],
                        },
                    ],
                    "palettes": [
                        {
                            "properties": [
                                { "name": "baseColor", "valuePool": 1 },
                            ],
                            "materials": [[0], [1]],
                        },
                        {
                            "properties": [
                                { "name": "emissiveStrength", "valuePool": 0 },
                            ],
                            "materials": [[1]],
                        },
                    ],
                    "objects": [
                        {
                            "name": "o",
                            "bounds": [2, 1, 1],
                            "origin": [0, 0, 0],
                            "voxelPositions": {
                                "encoding": "raw-json",
                                "data": [[0, 0, 0], [1, 0, 0]],
                            },
                            "layers": [0, 1],
                            "voxelSamples": {
                                "encoding": "raw-json",
                                "data": [[0, 1], [0, 0]],
                            },
                        },
                    ],
                    "nodes": [
                        {
                            "name": "o",
                            "transform": {
                                "position": [0.0, 0.0, 0.0],
                                "rotation": [0.0, 0.0, 0.0, 1.0],
                                "scale": [1.0, 1.0, 1.0],
                            },
                            "childNodes": [],
                            "childObjects": [0],
                        },
                    ],
                    "rootNodes": [0],
                },
                "ext": { "vendor": null },
            },
        })
    }

    #[test]
    fn round_trips_the_wire_shape() {
        let file = document();
        let wire = serde_json::to_value(&file).unwrap();
        assert_eq!(wire, wire_document());
        assert_eq!(serde_json::from_value::<VoxjFile>(wire).unwrap(), file);
    }

    #[test]
    fn null_optional_fields_reject() {
        for key in ["editState", "ext"] {
            let mut wire = wire_document();
            wire.pointer_mut("/main")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert(key.to_owned(), Value::Null);
            assert!(serde_json::from_value::<VoxjFile>(wire).is_err());
        }
    }

    #[test]
    fn unknown_fields_reject() {
        let mut wire = wire_document();
        wire.pointer_mut("/main")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_owned(), json!([]));
        assert!(serde_json::from_value::<VoxjFile>(wire).is_err());
    }
}
