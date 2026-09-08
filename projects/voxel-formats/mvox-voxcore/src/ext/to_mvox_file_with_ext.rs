use crate::{Result, ext::MVoxVoxMain, write_mvox};
use mvox::MVoxFile;

/// Writes a [`MVoxVoxMain`] back to a decoded MagicaVoxel [`MVoxFile`], the
/// inverse of [`from_mvox_file_with_ext`](crate::ext::from_mvox_file_with_ext)
/// and the typed form of [`to_mvox_file`](crate::to_mvox_file). A loaded file
/// writes back exactly through its ext. A state carrying none writes a
/// synthesized file.
///
/// Errors if the ext's per-node entries do not line up with the hierarchy.
pub fn to_mvox_file_with_ext(state: &MVoxVoxMain) -> Result<MVoxFile> {
    write_mvox(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{from_mvox_file_with_ext, to_mvox_file_with_ext};
    use mvox::{
        MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer,
        MVoxMaterial, MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette,
        MVoxRenderObject, MVoxRotation, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel,
        MVoxShapeNode, MVoxTransformNode, MVoxUnknownChunk, MVoxVoxel,
    };
    use std::{array, collections::BTreeSet};
    use voxcore::{VoxValuePoolValueRef, material::IOR};

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    /// A file exercising every modeled chunk: two models, a custom palette, a
    /// transform -> group -> shape chain, a material, a layer, a render object,
    /// a camera, palette notes, an index map, and an unknown chunk. The
    /// reserved color 0 is left transparent so the file is also byte-stable.
    fn sample_file() -> MVoxFile {
        let mut colors = MVoxPalette::default().colors;
        colors[1] = MVoxColor::new(10, 20, 30, 40);
        colors[255] = MVoxColor::new(1, 2, 3, 4);

        MVoxFile {
            version: 150,
            models: vec![
                MVoxModel {
                    size: [2, 1, 1],
                    voxels: vec![
                        MVoxVoxel {
                            x: 0,
                            y: 0,
                            z: 0,
                            color_index: 1,
                        },
                        MVoxVoxel {
                            x: 1,
                            y: 0,
                            z: 0,
                            color_index: 255,
                        },
                    ],
                },
                MVoxModel {
                    size: [1, 1, 1],
                    voxels: vec![MVoxVoxel {
                        x: 0,
                        y: 0,
                        z: 0,
                        color_index: 2,
                    }],
                },
            ],
            palette: Some(MVoxPalette { colors }),
            scene_nodes: vec![
                MVoxSceneNode {
                    id: 0,
                    attributes: MVoxNodeAttributes {
                        name: Some("root".to_owned()),
                        hidden: Some(false),
                        extra: MVoxDict(vec![pair("_meta", "x")]),
                    },
                    body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
                        child: 1,
                        layer: -1,
                        frames: vec![
                            MVoxFrame {
                                rotation: MVoxRotation(105),
                                translation: [1, -2, 3],
                                frame_index: Some(0),
                                extra: MVoxDict(vec![pair("_custom", "1")]),
                            },
                            MVoxFrame::default(),
                        ],
                    }),
                },
                MVoxSceneNode {
                    id: 1,
                    attributes: MVoxNodeAttributes::default(),
                    body: MVoxSceneNodeBody::Group(MVoxGroupNode { children: vec![2] }),
                },
                MVoxSceneNode {
                    id: 2,
                    attributes: MVoxNodeAttributes {
                        name: Some("shape".to_owned()),
                        hidden: Some(true),
                        extra: MVoxDict::default(),
                    },
                    body: MVoxSceneNodeBody::Shape(MVoxShapeNode {
                        models: vec![MVoxShapeModel {
                            model: 0,
                            frame_index: Some(0),
                            extra: MVoxDict(vec![pair("_x", "y")]),
                        }],
                    }),
                },
            ],
            materials: vec![MVoxMaterial {
                id: 1,
                material_type: Some(MVoxMaterialType::Glass),
                weight: Some(0.5),
                rough: Some(0.1),
                spec: None,
                ior: Some(1.5),
                att: None,
                flux: None,
                extra: MVoxDict(vec![pair("_g", "0.8")]),
            }],
            layers: vec![MVoxLayer {
                id: 0,
                name: Some("Layer 0".to_owned()),
                hidden: None,
                extra: MVoxDict::default(),
            }],
            render_objects: vec![MVoxRenderObject {
                attributes: MVoxDict(vec![pair("_type", "_ground"), pair("_color", "1 1 1")]),
            }],
            cameras: vec![MVoxCamera {
                id: 0,
                mode: Some("pers".to_owned()),
                focus: Some([1.0, 2.5, -3.0]),
                angle: Some([0.0, 0.0, 0.0]),
                radius: Some(8),
                frustum: Some(0.25),
                fov: Some(45),
                extra: MVoxDict(vec![pair("_aperture", "0")]),
            }],
            palette_notes: vec!["red".to_owned(), "green".to_owned()],
            index_map: Some(array::from_fn(|i| i as u8)),
            unknown_chunks: vec![MVoxUnknownChunk {
                id: *b"MATT",
                content: vec![1, 2, 3, 4],
                children: Vec::new(),
            }],
        }
    }

    /// The `(position, color)` set of a model's voxels, order-independent: the
    /// dense grid re-emits voxels in raster order rather than stored order.
    fn voxel_set(model: &MVoxModel) -> BTreeSet<(u8, u8, u8, u8)> {
        model
            .voxels
            .iter()
            .map(|voxel| (voxel.x, voxel.y, voxel.z, voxel.color_index))
            .collect()
    }

    fn assert_files_eq(got: &MVoxFile, want: &MVoxFile) {
        assert_eq!(got.version, want.version);
        assert_eq!(got.palette, want.palette);
        assert_eq!(got.scene_nodes, want.scene_nodes);
        assert_eq!(got.materials, want.materials);
        assert_eq!(got.layers, want.layers);
        assert_eq!(got.render_objects, want.render_objects);
        assert_eq!(got.cameras, want.cameras);
        assert_eq!(got.palette_notes, want.palette_notes);
        assert_eq!(got.index_map, want.index_map);
        assert_eq!(got.unknown_chunks, want.unknown_chunks);
        assert_eq!(got.models.len(), want.models.len());
        for (got, want) in got.models.iter().zip(&want.models) {
            assert_eq!(got.size, want.size);
            assert_eq!(voxel_set(got), voxel_set(want));
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_mvox_file_with_ext(&file).unwrap();
        assert_files_eq(&to_mvox_file_with_ext(&state).unwrap(), &file);
    }

    #[test]
    fn round_trips_the_empty_file() {
        let file = MVoxFile::default();
        let state = from_mvox_file_with_ext(&file).unwrap();
        assert_eq!(to_mvox_file_with_ext(&state).unwrap(), file);
    }

    /// A file with no `RGBA` chunk keeps `palette: None`: the colors come from
    /// the built-in palette and no chunk is written back, but the voxel color
    /// indices still round-trip.
    #[test]
    fn round_trips_a_file_without_a_palette() {
        let file = MVoxFile {
            models: vec![MVoxModel {
                size: [2, 1, 1],
                voxels: vec![MVoxVoxel {
                    x: 1,
                    y: 0,
                    z: 0,
                    color_index: 7,
                }],
            }],
            ..Default::default()
        };
        let state = from_mvox_file_with_ext(&file).unwrap();
        let rebuilt = to_mvox_file_with_ext(&state).unwrap();
        assert!(rebuilt.palette.is_none());
        assert_files_eq(&rebuilt, &file);
    }

    /// The codec accepts a non-finite scalar. A float value pool rejects NaN
    /// but holds the infinities, and the ext keeps the exact value either way,
    /// so the material still round-trips.
    #[test]
    fn round_trips_a_non_finite_material_scalar() {
        let file = MVoxFile {
            materials: vec![MVoxMaterial {
                id: 1,
                ior: Some(f32::INFINITY),
                ..Default::default()
            }],
            ..Default::default()
        };
        let state = from_mvox_file_with_ext(&file).unwrap();
        assert_files_eq(&to_mvox_file_with_ext(&state).unwrap(), &file);

        // The infinity reaches the value pool rather than defaulting: the wire
        // spells it and voxcore holds it.
        let (_, palette) = state.iter_palettes().next().unwrap();
        let property_id = palette.property_id_by_name(IOR).unwrap();
        let value_pool_id = palette.property(property_id).unwrap().value_pool_id;
        let value_pool = state.value_pool(value_pool_id).unwrap();
        assert!(
            value_pool
                .iter_values()
                .filter_map(|(value_id, _)| value_pool.value(value_id))
                .any(|value| value == VoxValuePoolValueRef::Float(f64::INFINITY)),
            "the infinite ior defaulted away"
        );
    }

    /// A shape node may draw the same model on several animation frames,
    /// listing the model index more than once. The voxcore node lists the
    /// placed object once, but the ext keeps the full list, so the shape
    /// round-trips.
    #[test]
    fn round_trips_a_shape_drawing_one_model_on_two_frames() {
        let file = MVoxFile {
            models: vec![MVoxModel {
                size: [1, 1, 1],
                voxels: vec![MVoxVoxel {
                    x: 0,
                    y: 0,
                    z: 0,
                    color_index: 1,
                }],
            }],
            scene_nodes: vec![MVoxSceneNode {
                id: 0,
                attributes: MVoxNodeAttributes::default(),
                body: MVoxSceneNodeBody::Shape(MVoxShapeNode {
                    models: vec![
                        MVoxShapeModel {
                            model: 0,
                            frame_index: Some(0),
                            extra: MVoxDict::default(),
                        },
                        MVoxShapeModel {
                            model: 0,
                            frame_index: Some(1),
                            extra: MVoxDict::default(),
                        },
                    ],
                }),
            }],
            ..Default::default()
        };
        let state = from_mvox_file_with_ext(&file).unwrap();
        assert_files_eq(&to_mvox_file_with_ext(&state).unwrap(), &file);
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_mvox_bytes_with_ext, to_mvox_bytes_with_ext};

        #[test]
        fn round_trips_through_mvox_bytes() {
            let file = sample_file();
            let state = from_mvox_file_with_ext(&file).unwrap();
            let bytes = to_mvox_bytes_with_ext(&state).unwrap();
            let reloaded = from_mvox_bytes_with_ext(&bytes).unwrap();
            assert_files_eq(&to_mvox_file_with_ext(&reloaded).unwrap(), &file);
        }
    }
}
