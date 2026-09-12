use crate::{Error, MVoxExt, MVoxExtFrame, MVoxExtNode, MVoxExtNodeBody, MVoxVoxMain, Result};
use branded_id::U32Id;
use mvox::{
    MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRenderObject, MVoxRotation,
    MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode, MVoxTransformNode,
    MVoxUnknownChunk, MVoxVoxel,
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxMain, VoxObject,
    color::value_pool_color, material::BASE_COLOR,
};

/// Writes a [`MVoxVoxMain`] to a decoded MagicaVoxel [`MVoxFile`], the
/// inverse of [`from_mvox_file`](crate::from_mvox_file). Each object emits one
/// model and the rest comes from the ext, so a loaded file rebuilds exactly
/// and a state [`to_mvox_vox_main`](crate::to_mvox_vox_main) gave its ext
/// writes as a file synthesized from the scene. A model lists its voxels in
/// ascending raster order, which need not match their original stored order.
///
/// Errors if the ext's scene nodes are out of step with the hierarchy, or if
/// a model exceeds a MagicaVoxel limit such as the per-axis voxel cap. A node
/// retained after the load has no scene node and counts as out of step.
pub fn to_mvox_file(state: &MVoxVoxMain) -> Result<MVoxFile> {
    let ext = state.ext();

    // The loader adds exactly one palette and references it from every
    // object on one layer. Each material's color resolves through its value
    // pool.
    let palette_id = state
        .iter_palettes()
        .next()
        .map(|(palette_id, _)| palette_id);
    let file_palette = ext.palette_present.then(|| MVoxPalette {
        colors: colors_from_palette(state, palette_id),
    });

    let materials = build_materials(ext);
    // Each object is the author's build volume, so the written model keeps
    // its dimensions and voxel positions directly.
    let models = state
        .iter_objects()
        .map(|(_, object)| model_from_object(object))
        .collect::<Result<Vec<_>>>()?;
    let scene_nodes = build_scene_nodes(state, ext)?;

    let layers = ext
        .layers
        .iter()
        .map(|layer| MVoxLayer {
            id: layer.id,
            name: layer.name.clone(),
            hidden: layer.hidden,
            extra: MVoxDict(layer.extra.clone()),
        })
        .collect();
    let render_objects = ext
        .render_objects
        .iter()
        .map(|attributes| MVoxRenderObject {
            attributes: MVoxDict(attributes.clone()),
        })
        .collect();
    let cameras = ext
        .cameras
        .iter()
        .map(|camera| MVoxCamera {
            id: camera.id,
            mode: camera.mode.clone(),
            focus: camera.focus,
            angle: camera.angle,
            radius: camera.radius,
            frustum: camera.frustum,
            fov: camera.fov,
            extra: MVoxDict(camera.extra.clone()),
        })
        .collect();
    let unknown_chunks = ext
        .unknown_chunks
        .iter()
        .map(|chunk| MVoxUnknownChunk {
            id: chunk.id,
            content: chunk.content.clone(),
            children: chunk.children.clone(),
        })
        .collect();

    Ok(MVoxFile {
        version: ext.version,
        models,
        palette: file_palette,
        scene_nodes,
        materials,
        layers,
        render_objects,
        cameras,
        palette_notes: ext.palette_notes.clone(),
        index_map: ext
            .index_map
            .as_deref()
            .map(|map| {
                <[u8; 256]>::try_from(map).map_err(|_| {
                    Error::Invalid(format!(
                        "mvox ext index map has {} entries, need exactly 256",
                        map.len()
                    ))
                })
            })
            .transpose()?,
        unknown_chunks,
    })
}

/// The 256 palette colors read back through `baseColor`: material `index`
/// gives color index `index`. Transparent where the palette or its color
/// property is absent.
fn colors_from_palette<T>(
    state: &VoxMain<T>,
    palette_id: Option<U32Id<BVoxPalette>>,
) -> [MVoxColor; 256] {
    let mut colors = [MVoxColor::default(); 256];
    let Some(palette_id) = palette_id else {
        return colors;
    };
    let Some(property_id) = state
        .palette(palette_id)
        .and_then(|palette| palette.property_id_by_name(BASE_COLOR))
    else {
        return colors;
    };
    for (index, color) in colors.iter_mut().enumerate() {
        let material_id = U32Id::<BVoxMaterial>::from_u32(index as u32);
        if let Some([r, g, b, a]) = state
            .material_value(palette_id, material_id, property_id)
            .and_then(|(value_pool, value_id)| value_pool_color(value_pool, value_id))
        {
            *color = MVoxColor::new(r, g, b, a);
        }
    }
    colors
}

/// Rebuilds the materials from the ext, which holds each one's exact optional
/// fields, so an absent field round-trips as absent. The palette's value pools
/// carry only a default-substituted neutral copy.
fn build_materials(ext: &MVoxExt) -> Vec<MVoxMaterial> {
    ext.materials
        .iter()
        .map(|material| MVoxMaterial {
            id: material.id,
            material_type: material
                .material_type
                .as_deref()
                .map(material_type_from_token),
            weight: material.weight,
            rough: material.rough,
            spec: material.spec,
            ior: material.ior,
            att: material.att,
            flux: material.flux,
            extra: MVoxDict(material.extra.clone()),
        })
        .collect()
}

/// The material shading model for a `_type` token. Known tokens map to their
/// variants; anything else is preserved as-is.
fn material_type_from_token(token: &str) -> MVoxMaterialType {
    match token {
        "_diffuse" => MVoxMaterialType::Diffuse,
        "_metal" => MVoxMaterialType::Metal,
        "_glass" => MVoxMaterialType::Glass,
        "_emit" => MVoxMaterialType::Emit,
        other => MVoxMaterialType::Other(other.to_owned()),
    }
}

/// Builds a model from an object: its size from the grid bounds and one voxel
/// per live cell, in ascending raster order, each taking its color index from
/// the material it samples on the object's first layer. A material's index is
/// its color index.
///
/// Errors when a material index or a voxel coordinate does not fit the byte
/// MagicaVoxel stores it in, since a wrapped index paints the voxel from the
/// wrong palette entry and a wrapped coordinate moves it.
fn model_from_object(object: &VoxObject) -> Result<MVoxModel> {
    let bounds = object.bounds();
    let layer_id = object.iter_layers().next().map(|(layer_id, _)| layer_id);

    let voxels = object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let color_index =
                match layer_id.and_then(|layer_id| object.voxel_material(voxel_id, layer_id)) {
                    Some(material_id) => model_byte(material_id.to_u32(), "material index")?,
                    None => 0,
                };
            Ok(MVoxVoxel {
                x: model_byte(position.x, "voxel x")?,
                y: model_byte(position.y, "voxel y")?,
                z: model_byte(position.z, "voxel z")?,
                color_index,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(MVoxModel {
        size: [bounds.x, bounds.y, bounds.z],
        voxels,
    })
}

/// `value` as the byte a MagicaVoxel model stores, or the error naming `label`.
fn model_byte(value: u32, label: &str) -> Result<u8> {
    u8::try_from(value).map_err(|_| {
        Error::Invalid(format!(
            "{label} {value} does not fit the byte a MagicaVoxel model stores"
        ))
    })
}

/// Rebuilds the scene nodes from the ext, one per entry in stored order. The
/// references come from the ext, which holds the exact lists, so a shape that
/// draws one model on several frames or any other repeated reference
/// round-trips. Errors if the ext is out of step with the hierarchy.
fn build_scene_nodes<T>(state: &VoxMain<T>, ext: &MVoxExt) -> Result<Vec<MVoxSceneNode>> {
    let node_count = state.hierarchy_node_count();
    if node_count != ext.scene_nodes.len() {
        return Err(Error::Invalid(format!(
            "mvox ext has {} scene nodes but the state has {node_count} hierarchy nodes",
            ext.scene_nodes.len()
        )));
    }

    let entries = ext
        .scene_nodes
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            entry.as_ref().ok_or_else(|| {
                Error::Invalid(format!(
                    "hierarchy node {index} was retained after the load and has no mvox scene node"
                ))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    check_references(state, &entries)?;

    Ok(entries
        .iter()
        .map(|provenance| MVoxSceneNode {
            id: provenance.id,
            attributes: MVoxNodeAttributes {
                name: provenance.name.clone(),
                hidden: provenance.hidden,
                extra: MVoxDict(provenance.attr_extra.clone()),
            },
            body: match &provenance.body {
                MVoxExtNodeBody::Transform {
                    child,
                    layer,
                    frames,
                } => MVoxSceneNodeBody::Transform(MVoxTransformNode {
                    child: *child,
                    layer: *layer,
                    frames: frames.iter().map(frame_from_provenance).collect(),
                }),
                MVoxExtNodeBody::Group { children } => MVoxSceneNodeBody::Group(MVoxGroupNode {
                    children: children.clone(),
                }),
                MVoxExtNodeBody::Shape { models } => MVoxSceneNodeBody::Shape(MVoxShapeNode {
                    models: models
                        .iter()
                        .map(|model| MVoxShapeModel {
                            model: model.model,
                            frame_index: model.frame_index,
                            extra: MVoxDict(model.extra.clone()),
                        })
                        .collect(),
                }),
            },
        })
        .collect())
}

/// Errors when an entry's distinct references differ from its node's children,
/// paired through the listings: a transform's child and a group's children by
/// scene-node id, a shape's models by object listing index.
fn check_references<T>(state: &VoxMain<T>, entries: &[&MVoxExtNode]) -> Result<()> {
    let id_by_node: HashMap<U32Id<BVoxHierarchyNode>, i32> = state
        .iter_hierarchy_nodes()
        .zip(entries)
        .map(|((node_id, _), entry)| (node_id, entry.id))
        .collect();
    let index_by_object: HashMap<U32Id<BVoxObject>, u32> = state
        .iter_objects()
        .enumerate()
        .map(|(index, (object_id, _))| (object_id, index as u32))
        .collect();

    for (index, ((_, node), entry)) in state.iter_hierarchy_nodes().zip(entries).enumerate() {
        let placed_ids: Vec<i32> = node
            .child_node_ids
            .iter()
            .map(|child_id| {
                *id_by_node
                    .get(child_id)
                    .expect("a child node is one of the state's")
            })
            .collect();
        let placed_indices: Vec<u32> = node
            .child_object_ids
            .iter()
            .map(|object_id| {
                *index_by_object
                    .get(object_id)
                    .expect("a placed object is one of the state's")
            })
            .collect();
        let (referenced_ids, referenced_indices) = match &entry.body {
            MVoxExtNodeBody::Transform { child, .. } => (vec![*child], Vec::new()),
            MVoxExtNodeBody::Group { children } => (distinct(children.iter().copied()), Vec::new()),
            MVoxExtNodeBody::Shape { models } => {
                (Vec::new(), distinct(models.iter().map(|model| model.model)))
            }
        };
        if referenced_ids != placed_ids || referenced_indices != placed_indices {
            return Err(Error::Invalid(format!(
                "mvox ext scene node {index} references nodes {referenced_ids:?} and models \
                 {referenced_indices:?} but hierarchy node {index} places nodes {placed_ids:?} \
                 and objects {placed_indices:?}"
            )));
        }
    }

    Ok(())
}

/// `values` without repeats, in first-seen order.
fn distinct<T: Copy + Eq + Hash>(values: impl Iterator<Item = T>) -> Vec<T> {
    let mut seen = HashSet::new();
    values.filter(|value| seen.insert(*value)).collect()
}

/// Rebuilds one transform-node frame from its ext provenance.
fn frame_from_provenance(frame: &MVoxExtFrame) -> MVoxFrame {
    MVoxFrame {
        rotation: MVoxRotation(frame.rotation),
        translation: frame.translation,
        frame_index: frame.frame_index,
        extra: MVoxDict(frame.extra.clone()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{MVoxExtNode, MVoxExtNodeBody, MVoxVoxMain, from_mvox_file, to_mvox_file};
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
        let state = from_mvox_file(&file).unwrap();
        assert_files_eq(&to_mvox_file(&state).unwrap(), &file);
    }

    #[test]
    fn round_trips_the_empty_file() {
        let file = MVoxFile::default();
        let state = from_mvox_file(&file).unwrap();
        assert_eq!(to_mvox_file(&state).unwrap(), file);
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
        let state = from_mvox_file(&file).unwrap();
        let rebuilt = to_mvox_file(&state).unwrap();
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
        let state = from_mvox_file(&file).unwrap();
        assert_files_eq(&to_mvox_file(&state).unwrap(), &file);

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
        let state = from_mvox_file(&file).unwrap();
        assert_files_eq(&to_mvox_file(&state).unwrap(), &file);
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
        let mut state = from_mvox_file(&file).unwrap();
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

        let rebuilt = to_mvox_file(&state).unwrap();
        let mut want = file;
        want.models.remove(1);
        want.scene_nodes.drain(4..6);
        want.scene_nodes[1].body = MVoxSceneNodeBody::Group(MVoxGroupNode {
            children: vec![2, 6],
        });
        want.scene_nodes[5] = shape_node(7, 1);
        assert_files_eq(&rebuilt, &want);

        let reloaded = from_mvox_file(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node retained after the load has no scene node, so the write errors.
    /// Releasing it restores the alignment. The file then writes again.
    #[test]
    fn a_node_retained_after_the_load_errors_until_released() {
        let file = placed_models_file();
        let mut state = from_mvox_file(&file).unwrap();
        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(state.ext().scene_nodes[8], None);
        assert!(to_mvox_file(&state).is_err());

        state.release_hierarchy_node(node_id).unwrap();
        assert_files_eq(&to_mvox_file(&state).unwrap(), &file);
    }

    /// Moving the last model to the front renumbers every shape's model
    /// index, so each shape still draws the model it did.
    #[test]
    fn a_moved_object_keeps_each_shapes_model() {
        let mut state = from_mvox_file(&placed_models_file()).unwrap();
        state
            .move_object(U32Id::<BVoxObject>::from_u32(2), 0)
            .unwrap();

        let rebuilt = to_mvox_file(&state).unwrap();
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
        let mut state = from_mvox_file(&file).unwrap();

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

        let rebuilt = to_mvox_file(&state).unwrap();
        assert_eq!(rebuilt.materials[2].id, 4);
        assert_eq!(rebuilt.materials[2].ior, Some(1.5));
    }

    /// An ext out of step with the hierarchy is malformed, so the writer
    /// errors instead of pairing entries by a shifted index or placing what
    /// the state does not.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let file = placed_models_file();
        let mut state = from_mvox_file(&file).unwrap();
        let ext = state.ext_mut();
        ext.scene_nodes.pop();
        assert!(to_mvox_file(&state).is_err());

        let mut state = from_mvox_file(&file).unwrap();
        let ext = state.ext_mut();
        let Some(MVoxExtNode {
            body: MVoxExtNodeBody::Group { children },
            ..
        }) = &mut ext.scene_nodes[1]
        else {
            panic!("node 1 is the group");
        };
        children.push(3);
        assert!(to_mvox_file(&state).is_err());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_mvox_bytes, to_mvox_bytes};

        #[test]
        fn round_trips_through_mvox_bytes() {
            let file = sample_file();
            let state = from_mvox_file(&file).unwrap();
            let bytes = to_mvox_bytes(&state).unwrap();
            let reloaded = from_mvox_bytes(&bytes).unwrap();
            assert_files_eq(&to_mvox_file(&reloaded).unwrap(), &file);
        }
    }
}
