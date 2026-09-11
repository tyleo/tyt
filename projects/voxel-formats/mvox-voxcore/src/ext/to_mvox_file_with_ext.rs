use crate::{MVoxExtSource, Result, ext::MVoxVoxMain};
use mvox::MVoxFile;

/// Writes a [`MVoxVoxMain`] back to a decoded MagicaVoxel [`MVoxFile`], the
/// inverse of [`from_mvox_file_with_ext`](crate::ext::from_mvox_file_with_ext)
/// and the typed form of [`to_mvox_file`](crate::to_mvox_file). A loaded file
/// writes back exactly through its ext.
///
/// Errors if the ext is out of step with the hierarchy. A node retained after
/// the load has no scene node and counts as out of step.
pub fn to_mvox_file_with_ext(state: &MVoxVoxMain) -> Result<MVoxFile> {
    MVoxExtSource::write_mvox(state)
}

#[cfg(test)]
mod tests {
    use crate::ext::{
        MVoxExtNode, MVoxExtNodeBody, MVoxVoxMain, from_mvox_file_with_ext, to_mvox_file_with_ext,
    };
    use branded_id::U32Id;
    use mvox::{
        MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer,
        MVoxMaterial, MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette,
        MVoxRenderObject, MVoxRotation, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel,
        MVoxShapeNode, MVoxTransformNode, MVoxUnknownChunk, MVoxVoxel,
    };
    use std::{array, collections::BTreeSet};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode,
        VoxValuePoolValueRef, material::IOR,
    };

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

    /// One 1x1x1 model of `color_index`.
    fn unit_model(color_index: u8) -> MVoxModel {
        MVoxModel {
            size: [1, 1, 1],
            voxels: vec![MVoxVoxel {
                x: 0,
                y: 0,
                z: 0,
                color_index,
            }],
        }
    }

    /// A named transform node over `child`.
    fn transform_node(id: i32, child: i32, name: &str) -> MVoxSceneNode {
        MVoxSceneNode {
            id,
            attributes: MVoxNodeAttributes {
                name: Some(name.to_owned()),
                ..Default::default()
            },
            body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
                child,
                layer: -1,
                frames: vec![MVoxFrame::default()],
            }),
        }
    }

    /// A shape node drawing `model` on its first frame.
    fn shape_node(id: i32, model: u32) -> MVoxSceneNode {
        MVoxSceneNode {
            id,
            attributes: MVoxNodeAttributes::default(),
            body: MVoxSceneNodeBody::Shape(MVoxShapeNode {
                models: vec![MVoxShapeModel {
                    model,
                    frame_index: Some(0),
                    extra: MVoxDict::default(),
                }],
            }),
        }
    }

    /// Three models placed under one root: a transform over a group of three
    /// transform -> shape chains, each shape drawing one model.
    fn placed_models_file() -> MVoxFile {
        MVoxFile {
            models: vec![unit_model(1), unit_model(2), unit_model(3)],
            scene_nodes: vec![
                transform_node(0, 1, "root"),
                MVoxSceneNode {
                    id: 1,
                    attributes: MVoxNodeAttributes::default(),
                    body: MVoxSceneNodeBody::Group(MVoxGroupNode {
                        children: vec![2, 4, 6],
                    }),
                },
                transform_node(2, 3, "a"),
                shape_node(3, 0),
                transform_node(4, 5, "b"),
                shape_node(5, 1),
                transform_node(6, 7, "c"),
                shape_node(7, 2),
            ],
            ..Default::default()
        }
    }

    /// Replaces the children of hierarchy node `index`.
    fn set_children(
        state: &mut MVoxVoxMain,
        index: u32,
        child_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
        child_object_ids: Vec<U32Id<BVoxObject>>,
    ) {
        let node_id = U32Id::<BVoxHierarchyNode>::from_u32(index);
        let node = VoxHierarchyNode {
            child_node_ids,
            child_object_ids,
            ..state.hierarchy_node(node_id).unwrap().clone()
        };
        state.set_hierarchy_node(node_id, node).unwrap();
    }

    /// Dropping the middle placement, its two nodes, and its model, then
    /// compacting, leaves the survivors' provenance aligned: the group lists
    /// the surviving transforms, the last shape draws its model at the new
    /// index, and the rebuilt file reloads to the loaded ext minus the
    /// released entries.
    #[test]
    fn released_entities_leave_the_survivors_provenance_aligned() {
        let file = placed_models_file();
        let mut state = from_mvox_file_with_ext(&file).unwrap();
        let original = state.ext().clone();

        // The group, node 1, lists transforms 2, 4, and 6. Transform 4 places
        // shape 5, which draws model 1.
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        set_children(&mut state, 1, vec![node(2), node(6)], Vec::new());
        set_children(&mut state, 4, Vec::new(), Vec::new());
        set_children(&mut state, 5, Vec::new(), Vec::new());
        state.release_hierarchy_node(node(4)).unwrap();
        state.release_hierarchy_node(node(5)).unwrap();
        state
            .release_object(U32Id::<BVoxObject>::from_u32(1))
            .unwrap();
        state.gc();

        let mut expected = original;
        expected.scene_nodes.drain(4..6);
        let Some(MVoxExtNode {
            body: MVoxExtNodeBody::Group { children },
            ..
        }) = &mut expected.scene_nodes[1]
        else {
            panic!("node 1 is the group");
        };
        *children = vec![2, 6];
        let Some(MVoxExtNode {
            body: MVoxExtNodeBody::Shape { models },
            ..
        }) = &mut expected.scene_nodes[5]
        else {
            panic!("node 7 is the last shape");
        };
        models[0].model = 1;
        assert_eq!(state.ext(), &expected.clone());

        let rebuilt = to_mvox_file_with_ext(&state).unwrap();
        let mut want = file;
        want.models.remove(1);
        want.scene_nodes.drain(4..6);
        want.scene_nodes[1].body = MVoxSceneNodeBody::Group(MVoxGroupNode {
            children: vec![2, 6],
        });
        want.scene_nodes[5] = shape_node(7, 1);
        assert_files_eq(&rebuilt, &want);

        let reloaded = from_mvox_file_with_ext(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node retained after the load has no scene node, so the write errors.
    /// Releasing it restores the alignment. The file then writes again.
    #[test]
    fn a_node_retained_after_the_load_errors_until_released() {
        let file = placed_models_file();
        let mut state = from_mvox_file_with_ext(&file).unwrap();
        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(state.ext().scene_nodes[8], None);
        assert!(to_mvox_file_with_ext(&state).is_err());

        state.release_hierarchy_node(node_id).unwrap();
        assert_files_eq(&to_mvox_file_with_ext(&state).unwrap(), &file);
    }

    /// Moving the last model to the front renumbers every shape's model
    /// index, so each shape still draws the model it did.
    #[test]
    fn a_moved_object_keeps_each_shapes_model() {
        let mut state = from_mvox_file_with_ext(&placed_models_file()).unwrap();
        state
            .move_object(U32Id::<BVoxObject>::from_u32(2), 0)
            .unwrap();

        let rebuilt = to_mvox_file_with_ext(&state).unwrap();
        let models: Vec<u32> = rebuilt
            .scene_nodes
            .iter()
            .filter_map(|node| match &node.body {
                MVoxSceneNodeBody::Shape(shape) => Some(shape.models[0].model),
                _ => None,
            })
            .collect();
        assert_eq!(models, [1, 2, 0]);
        assert_eq!(rebuilt.models[0].voxels[0].color_index, 3);
    }

    /// Releasing a material shifts the recorded ids above it down with the
    /// palette's compaction. The rebuilt file records each survivor under the
    /// id its voxels sample.
    #[test]
    fn a_released_material_shifts_the_recorded_ids() {
        let mut file = placed_models_file();
        file.materials = vec![
            MVoxMaterial {
                id: 1,
                weight: Some(0.5),
                ..Default::default()
            },
            MVoxMaterial {
                id: 2,
                rough: Some(0.25),
                ..Default::default()
            },
            MVoxMaterial {
                id: 5,
                ior: Some(1.5),
                ..Default::default()
            },
        ];
        let mut state = from_mvox_file_with_ext(&file).unwrap();

        // No voxel samples material 4, so it releases without a repaint.
        state
            .release_material(
                U32Id::<BVoxPalette>::from_u32(0),
                U32Id::<BVoxMaterial>::from_u32(4),
            )
            .unwrap();
        state.gc();

        let ids: Vec<i32> = state
            .ext()
            .materials
            .iter()
            .map(|material| material.id)
            .collect();
        assert_eq!(ids, [1, 2, 4]);

        let rebuilt = to_mvox_file_with_ext(&state).unwrap();
        assert_eq!(rebuilt.materials[2].id, 4);
        assert_eq!(rebuilt.materials[2].ior, Some(1.5));
    }

    /// An ext out of step with the hierarchy is malformed, so the writer
    /// errors instead of pairing entries by a shifted index or placing what
    /// the state does not.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let file = placed_models_file();
        let mut state = from_mvox_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        ext.scene_nodes.pop();
        assert!(to_mvox_file_with_ext(&state).is_err());

        let mut state = from_mvox_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        let Some(MVoxExtNode {
            body: MVoxExtNodeBody::Group { children },
            ..
        }) = &mut ext.scene_nodes[1]
        else {
            panic!("node 1 is the group");
        };
        children.push(3);
        assert!(to_mvox_file_with_ext(&state).is_err());
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
