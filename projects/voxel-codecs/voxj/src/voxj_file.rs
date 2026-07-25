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
        VoxjBound, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjObject, VoxjPalette,
        VoxjPositionBlock, VoxjProperty, VoxjRuntimeState, VoxjSampleBlock, VoxjTransform,
        VoxjValuePool,
    };
    use serde_json::{Value, json};

    /// Covers the full palette surface: two palettes sharing a pool, and one
    /// channel per layer.
    fn document() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        VoxjValuePool::Float {
                            min: VoxjBound::None,
                            max: VoxjBound::None,
                            values: vec![1.0, 2.5],
                        },
                        VoxjValuePool::SrgbHex {
                            values: vec!["#FF0000".to_owned(), "#00FF00".to_owned()],
                        },
                    ],
                    palettes: vec![
                        VoxjPalette {
                            properties: vec![VoxjProperty {
                                name: "baseColorFactor".to_owned(),
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
                ext: None,
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
                        {
                            "kind": "float",
                            "min": "none",
                            "max": "none",
                            "values": [1.0, 2.5],
                        },
                        { "kind": "srgb-hex", "values": ["#FF0000", "#00FF00"] },
                    ],
                    "palettes": [
                        {
                            "properties": [
                                { "name": "baseColorFactor", "valuePool": 1 },
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
            },
        })
    }

    fn rename_key(value: &mut Value, pointer: &str, from: &str, to: &str) {
        let object = value.pointer_mut(pointer).unwrap().as_object_mut().unwrap();
        let inner = object.remove(from).unwrap();
        object.insert(to.to_owned(), inner);
    }

    #[test]
    fn round_trips_the_wire_shape() {
        let file = document();
        let wire = serde_json::to_value(&file).unwrap();
        assert_eq!(wire, wire_document());
        assert_eq!(serde_json::from_value::<VoxjFile>(wire).unwrap(), file);
    }

    #[test]
    fn old_wire_names_reject() {
        for (pointer, from, to) in [
            (
                "/main/runtimeState/palettes/0",
                "properties",
                "arrayProperties",
            ),
            ("/main/runtimeState/palettes/0", "properties", "bindings"),
            ("/main/runtimeState/objects/0", "layers", "layerPaletteRefs"),
            ("/main/runtimeState", "nodes", "hierarchyNodes"),
            ("/main/runtimeState", "rootNodes", "rootHierarchyNodes"),
        ] {
            let mut wire = wire_document();
            rename_key(&mut wire, pointer, from, to);
            assert!(serde_json::from_value::<VoxjFile>(wire).is_err());
        }
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
    fn non_object_ext_rejects() {
        for value in [json!(42), json!("x"), json!(true), json!([1])] {
            let mut wire = wire_document();
            wire.pointer_mut("/main")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("ext".to_owned(), value);
            assert!(serde_json::from_value::<VoxjFile>(wire).is_err());
        }
    }

    #[test]
    fn ext_nests_null() {
        let mut wire = wire_document();
        wire.pointer_mut("/main")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("ext".to_owned(), json!({ "vendor": null }));
        let file = serde_json::from_value::<VoxjFile>(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(&file).unwrap(), wire);
    }

    #[test]
    fn scalar_properties_reject() {
        let mut wire = wire_document();
        wire.pointer_mut("/main/runtimeState/palettes/0")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("scalarProperties".to_owned(), json!([]));
        assert!(serde_json::from_value::<VoxjFile>(wire).is_err());
    }
}
