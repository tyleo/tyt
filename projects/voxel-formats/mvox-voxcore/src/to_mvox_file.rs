use crate::{
    Error, MVoxExt, MVoxExtFrame, MVoxExtNode, MVoxExtNodeBody, MVoxVoxMain, Result,
    transform_from_frames,
};
use branded_id::U32Id;
use mvox::{
    MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer, MVoxMaterial,
    MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette, MVoxRenderObject, MVoxRotation,
    MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel, MVoxShapeNode, MVoxTransformNode,
    MVoxUnknownChunk, MVoxVoxel,
};
use std::collections::{HashMap, HashSet};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxObject,
    VoxState, color::value_pool_color, material::BASE_COLOR,
};

/// Colors a MagicaVoxel palette holds, so a material id past this has no
/// color slot.
const PALETTE_COLORS: u32 = 256;

/// How far a frame's projected rotation or scale may drift from the node's
/// before the two count as disagreeing.
const TRANSFORM_TOLERANCE: f64 = 1e-6;

/// Writes a [`MVoxVoxMain`] to a decoded MagicaVoxel [`MVoxFile`], the
/// inverse of [`from_mvox_file`](crate::from_mvox_file). Each object emits one
/// model. Each hierarchy node emits one scene node of the kind its ext entry
/// says, with its name and child links from the node and the rest from the
/// entry, so a loaded file rebuilds exactly and a state
/// [`to_mvox_vox_main`](crate::to_mvox_vox_main) gave its ext writes as a
/// file synthesized from the scene. A model lists its voxels in ascending
/// raster order, which need not match their original stored order. `MATL`
/// chunks write in material order.
///
/// Errors if:
///
/// 1. the state holds more than one palette, or a material id past 255
/// 2. a hierarchy node has no ext entry, an entry keys nothing, or two
///    entries share a scene-node id
/// 3. a node's children do not fit its entry's kind: a transform places one
///    child node and no objects, a group places no objects, a shape places
///    objects and no child nodes, and a shape entry draws exactly the
///    objects its node places
/// 4. a transform's first frame no longer projects to its node's transform
/// 5. a model exceeds a MagicaVoxel limit such as the per-axis voxel cap
pub fn to_mvox_file(main: &MVoxVoxMain) -> Result<MVoxFile> {
    let ext = main.ext();
    let state = main.state();

    let palette_id = check_palette(state)?;
    let file_palette = ext.palette_present.then(|| MVoxPalette {
        colors: colors_from_palette(state, palette_id),
    });

    let materials = build_materials(state, palette_id, ext)?;
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

/// The state's one palette, or `None` for a state with none. Errors on more
/// than one, since a MagicaVoxel file holds exactly one, and on a material
/// id past 255, which has no color slot.
fn check_palette(state: &VoxState) -> Result<Option<U32Id<BVoxPalette>>> {
    let count = state.palette_count();
    if count > 1 {
        return Err(Error::Invalid(format!(
            "the state holds {count} palettes but a MagicaVoxel file holds one"
        )));
    }

    let Some((palette_id, palette)) = state.iter_palettes().next() else {
        return Ok(None);
    };
    if let Some(material_id) = palette
        .iter_materials()
        .find(|material_id| material_id.to_u32() >= PALETTE_COLORS)
    {
        return Err(Error::Invalid(format!(
            "material {} has no slot in the {PALETTE_COLORS} colors a MagicaVoxel palette holds",
            material_id.to_u32()
        )));
    }

    Ok(Some(palette_id))
}

/// The 256 palette colors read back through `baseColor`: material `index`
/// gives color index `index`. Transparent where the palette or its color
/// property is absent.
fn colors_from_palette(
    state: &VoxState,
    palette_id: Option<U32Id<BVoxPalette>>,
) -> [MVoxColor; 256] {
    let mut colors = [MVoxColor::default(); 256];
    let Some(palette_id) = palette_id else {
        return colors;
    };
    let palette = state.palette(palette_id).expect("the palette is listed");
    let Some(property_id) = palette.property_id_by_name(BASE_COLOR) else {
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

/// Rebuilds the `MATL` chunks in material order from the ext, which holds
/// each one's exact optional fields, so an absent field round-trips as absent.
/// The palette's value pools carry only a default-substituted neutral copy.
/// Errors when an entry keys a material the palette does not hold.
fn build_materials(
    state: &VoxState,
    palette_id: Option<U32Id<BVoxPalette>>,
    ext: &MVoxExt,
) -> Result<Vec<MVoxMaterial>> {
    let palette = palette_id.and_then(|palette_id| state.palette(palette_id));
    let listed: Vec<U32Id<BVoxMaterial>> = palette
        .map(|palette| palette.iter_materials().collect())
        .unwrap_or_default();
    if let Some(material_id) = ext
        .materials
        .keys()
        .find(|material_id| !listed.contains(material_id))
    {
        return Err(Error::Invalid(format!(
            "mvox ext keeps material {} but the palette does not hold it",
            material_id.to_u32()
        )));
    }

    Ok(listed
        .into_iter()
        .filter_map(|material_id| {
            let material = ext.materials.get(&material_id)?;
            Some(MVoxMaterial {
                id: material_id.to_u32() as i32,
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
        })
        .collect())
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

/// Rebuilds the scene nodes, one per hierarchy node in listing order. The
/// name and the child links come from the node, the kind and the rest from
/// its entry. Errors if the ext is out of step with the hierarchy or a node's
/// children do not fit its kind.
fn build_scene_nodes(state: &VoxState, ext: &MVoxExt) -> Result<Vec<MVoxSceneNode>> {
    let node_count = state.hierarchy_node_count();
    if node_count != ext.scene_nodes.len() {
        return Err(Error::Invalid(format!(
            "mvox ext has {} scene nodes but the state has {node_count} hierarchy nodes",
            ext.scene_nodes.len()
        )));
    }

    let mut entries: Vec<(U32Id<BVoxHierarchyNode>, &VoxHierarchyNode, &MVoxExtNode)> =
        Vec::with_capacity(node_count);
    let mut scene_ids = HashSet::with_capacity(node_count);
    for (node_id, node) in state.iter_hierarchy_nodes() {
        let Some(entry) = ext.scene_nodes.get(&node_id) else {
            return Err(Error::Invalid(format!(
                "hierarchy node {} has no mvox scene node",
                node_id.to_u32()
            )));
        };
        if !scene_ids.insert(entry.id) {
            return Err(Error::Invalid(format!(
                "mvox scene node id {} is declared more than once",
                entry.id
            )));
        }
        entries.push((node_id, node, entry));
    }

    let entry_by_node: HashMap<U32Id<BVoxHierarchyNode>, &MVoxExtNode> = entries
        .iter()
        .map(|&(node_id, _, entry)| (node_id, entry))
        .collect();
    let index_by_object: HashMap<U32Id<BVoxObject>, u32> = state
        .iter_objects()
        .enumerate()
        .map(|(index, (object_id, _))| (object_id, index as u32))
        .collect();

    entries
        .iter()
        .map(|&(node_id, node, entry)| {
            let body = scene_node_body(node_id, node, entry, &entry_by_node, &index_by_object)?;
            Ok(MVoxSceneNode {
                id: entry.id,
                attributes: MVoxNodeAttributes {
                    name: (!node.name.is_empty()).then(|| node.name.clone()),
                    hidden: entry.hidden,
                    extra: MVoxDict(entry.attr_extra.clone()),
                },
                body,
            })
        })
        .collect()
}

/// The scene-node body of `node`, which is hierarchy node `node_id`, from its
/// `entry`. Errors when the node's children do not fit the entry's kind.
fn scene_node_body(
    node_id: U32Id<BVoxHierarchyNode>,
    node: &VoxHierarchyNode,
    entry: &MVoxExtNode,
    entry_by_node: &HashMap<U32Id<BVoxHierarchyNode>, &MVoxExtNode>,
    index_by_object: &HashMap<U32Id<BVoxObject>, u32>,
) -> Result<MVoxSceneNodeBody> {
    let id = node_id.to_u32();
    let child_entry = |child_id: &U32Id<BVoxHierarchyNode>| {
        *entry_by_node
            .get(child_id)
            .expect("a child node is one of the state's")
    };

    match &entry.body {
        MVoxExtNodeBody::Transform { layer, frames } => {
            let [child_id] = node.child_node_ids.as_slice() else {
                return Err(Error::Invalid(format!(
                    "hierarchy node {id} is a transform but places {} child nodes, not one",
                    node.child_node_ids.len()
                )));
            };
            if !node.child_object_ids.is_empty() {
                return Err(Error::Invalid(format!(
                    "hierarchy node {id} is a transform but places objects"
                )));
            }
            let child = child_entry(child_id);
            let frames: Vec<MVoxFrame> = frames.iter().map(frame_from_provenance).collect();
            check_frame(id, node, &frames)?;
            Ok(MVoxSceneNodeBody::Transform(MVoxTransformNode {
                child: child.id,
                layer: *layer,
                frames,
            }))
        }
        MVoxExtNodeBody::Group => {
            if !node.child_object_ids.is_empty() {
                return Err(Error::Invalid(format!(
                    "hierarchy node {id} is a group but places objects"
                )));
            }
            let children = node
                .child_node_ids
                .iter()
                .map(|child_id| child_entry(child_id).id)
                .collect();
            Ok(MVoxSceneNodeBody::Group(MVoxGroupNode { children }))
        }
        MVoxExtNodeBody::Shape { models } => {
            if !node.child_node_ids.is_empty() {
                return Err(Error::Invalid(format!(
                    "hierarchy node {id} is a shape but places child nodes"
                )));
            }
            let drawn: HashSet<U32Id<BVoxObject>> =
                models.iter().map(|model| model.object).collect();
            let placed: HashSet<U32Id<BVoxObject>> =
                node.child_object_ids.iter().copied().collect();
            if drawn != placed {
                let drawn: Vec<u32> = models.iter().map(|model| model.object.to_u32()).collect();
                let placed: Vec<u32> = node
                    .child_object_ids
                    .iter()
                    .map(|object_id| object_id.to_u32())
                    .collect();
                return Err(Error::Invalid(format!(
                    "mvox shape entry of hierarchy node {id} draws objects {drawn:?} but the node \
                     places {placed:?}"
                )));
            }
            Ok(MVoxSceneNodeBody::Shape(MVoxShapeNode {
                models: models
                    .iter()
                    .map(|model| MVoxShapeModel {
                        model: index_by_object[&model.object],
                        frame_index: model.frame_index,
                        extra: MVoxDict(model.extra.clone()),
                    })
                    .collect(),
            }))
        }
    }
}

/// Errors when `frames` no longer projects to the transform of hierarchy node
/// `id`, which is `node`: the node moved, turned, or scaled after the load
/// and its frames did not follow.
fn check_frame(id: u32, node: &VoxHierarchyNode, frames: &[MVoxFrame]) -> Result<()> {
    let projected = transform_from_frames(frames);
    let transform = &node.transform;
    let agrees = projected.position == transform.position
        && projected.rotation.dot(transform.rotation).abs() >= 1.0 - TRANSFORM_TOLERANCE
        && projected
            .scale
            .abs_diff_eq(transform.scale, TRANSFORM_TOLERANCE);
    if agrees {
        return Ok(());
    }

    Err(Error::Invalid(format!(
        "hierarchy node {id} has transform {transform:?} but its mvox frames project to \
         {projected:?}; edit the ext frames along with the node"
    )))
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
    use crate::{
        MVoxExtNode, MVoxExtNodeBody, MVoxExtShapeModel, MVoxVoxMain, from_mvox_file, to_mvox_file,
    };
    use branded_id::U32Id;
    use mvox::{
        MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer,
        MVoxMaterial, MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette,
        MVoxRenderObject, MVoxRotation, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel,
        MVoxShapeNode, MVoxTransformNode, MVoxUnknownChunk, MVoxVoxel,
    };
    use std::{
        array,
        collections::{BTreeMap, BTreeSet},
    };
    use ty_math::{TyTransformF64, TyVector3F64};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxPalette,
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

    fn node(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object(index: u32) -> U32Id<BVoxObject> {
        U32Id::from_u32(index)
    }

    fn material(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    #[test]
    fn round_trips_through_vox_main() {
        let file = sample_file();

        let main = from_mvox_file(&file).unwrap();

        assert_files_eq(&to_mvox_file(&main).unwrap(), &file);
    }

    #[test]
    fn round_trips_the_empty_file() {
        let file = MVoxFile::default();

        let main = from_mvox_file(&file).unwrap();

        assert_eq!(to_mvox_file(&main).unwrap(), file);
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

        let main = from_mvox_file(&file).unwrap();

        let rebuilt = to_mvox_file(&main).unwrap();

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

        let main = from_mvox_file(&file).unwrap();

        assert_files_eq(&to_mvox_file(&main).unwrap(), &file);

        // The infinity reaches the value pool rather than defaulting: the wire
        // spells it and voxcore holds it.
        let (_, palette) = main.iter_palettes().next().unwrap();

        let property_id = palette.property_id_by_name(IOR).unwrap();

        let value_pool_id = palette.property(property_id).unwrap().value_pool_id;

        let value_pool = main.value_pool(value_pool_id).unwrap();

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

        let main = from_mvox_file(&file).unwrap();

        assert_files_eq(&to_mvox_file(&main).unwrap(), &file);
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
        main: &mut MVoxVoxMain,
        index: u32,
        child_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
        child_object_ids: Vec<U32Id<BVoxObject>>,
    ) {
        let replacement = VoxHierarchyNode {
            child_node_ids,
            child_object_ids,
            ..main.hierarchy_node(node(index)).unwrap().clone()
        };

        main.set_hierarchy_node(node(index), replacement).unwrap();
    }

    /// Dropping the middle placement, its two nodes, and its model, then
    /// compacting, rekeys the survivors' provenance: the group lists the
    /// surviving transforms, the last shape draws its object under its new
    /// id, and the rebuilt file reloads to the loaded ext minus the released
    /// entries.
    #[test]
    fn released_entities_leave_the_survivors_provenance_rekeyed() {
        let file = placed_models_file();

        let mut main = from_mvox_file(&file).unwrap();

        let original = main.ext().clone();

        // The group, node 1, lists transforms 2, 4, and 6. Transform 4 places
        // shape 5, which draws model 1.
        set_children(&mut main, 1, vec![node(2), node(6)], Vec::new());

        set_children(&mut main, 4, Vec::new(), Vec::new());

        set_children(&mut main, 5, Vec::new(), Vec::new());

        main.release_hierarchy_node(node(4)).unwrap();

        main.release_hierarchy_node(node(5)).unwrap();

        main.release_object(object(1)).unwrap();

        main.gc().unwrap();

        let mut expected = original;

        expected.scene_nodes.remove(&node(4));

        expected.scene_nodes.remove(&node(5));

        let mut last = expected.scene_nodes.remove(&node(7)).unwrap();

        let MVoxExtNodeBody::Shape { models } = &mut last.body else {
            panic!("node 7 is the last shape");
        };

        models[0].object = object(1);

        expected.scene_nodes.insert(node(5), last);

        let transform = expected.scene_nodes.remove(&node(6)).unwrap();

        expected.scene_nodes.insert(node(4), transform);

        assert_eq!(main.ext(), &expected);

        let rebuilt = to_mvox_file(&main).unwrap();

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

    /// A node retained after the load takes a synthesized entry on the spot,
    /// under a fresh scene-node id, so the file writes with it.
    #[test]
    fn a_node_retained_after_the_load_writes_with_a_synthesized_entry() {
        let file = placed_models_file();

        let mut main = from_mvox_file(&file).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(
            main.ext().scene_nodes[&node_id],
            MVoxExtNode {
                id: 8,
                hidden: None,
                attr_extra: Vec::new(),
                body: MVoxExtNodeBody::Group,
            }
        );

        let rebuilt = to_mvox_file(&main).unwrap();

        assert_eq!(
            rebuilt.scene_nodes[8],
            MVoxSceneNode {
                id: 8,
                attributes: MVoxNodeAttributes {
                    name: Some("added".to_owned()),
                    ..Default::default()
                },
                body: MVoxSceneNodeBody::Group(MVoxGroupNode {
                    children: Vec::new()
                }),
            }
        );
    }

    /// Moving the last model to the front renumbers every shape's model
    /// index, so each shape still draws the model it did.
    #[test]
    fn a_moved_object_keeps_each_shapes_model() {
        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        main.move_object(object(2), 0).unwrap();

        let rebuilt = to_mvox_file(&main).unwrap();

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

    /// A file with materials `1`, `2`, and `5`.
    fn materials_file() -> MVoxFile {
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

        file
    }

    /// Releasing a material drops its entry, and `gc` rekeys the survivors
    /// with the palette's compaction. The rebuilt file records each survivor
    /// under the id its voxels sample.
    #[test]
    fn a_released_material_drops_its_entry_and_gc_rekeys_the_rest() {
        let mut main = from_mvox_file(&materials_file()).unwrap();

        // No voxel samples material 4, so it releases without a repaint.
        main.release_material(U32Id::<BVoxPalette>::from_u32(0), material(4))
            .unwrap();

        main.gc().unwrap();

        let mut ids: Vec<u32> = main.ext().materials.keys().map(|id| id.to_u32()).collect();

        ids.sort_unstable();

        assert_eq!(ids, [1, 2, 4]);

        let rebuilt = to_mvox_file(&main).unwrap();

        assert_eq!(rebuilt.materials[2].id, 4);

        assert_eq!(rebuilt.materials[2].ior, Some(1.5));
    }

    /// An index-map byte naming a released material keeps its index until
    /// `gc`, then takes the index the compaction freed, so the map stays a
    /// permutation and the bytes above the release shift down with their
    /// materials.
    #[test]
    fn the_index_map_follows_a_released_material_through_gc() {
        let mut file = materials_file();

        file.index_map = Some(array::from_fn(|i| (i as u8).wrapping_add(1)));

        let mut main = from_mvox_file(&file).unwrap();

        main.release_material(U32Id::<BVoxPalette>::from_u32(0), material(4))
            .unwrap();

        assert_eq!(to_mvox_file(&main).unwrap().index_map, file.index_map);

        main.gc().unwrap();

        let rebuilt = to_mvox_file(&main).unwrap();

        let map = rebuilt.index_map.unwrap();

        // Slot 3 named 4, the released material.
        assert_eq!(&map[..5], &[1, 2, 3, 255, 4]);

        assert_eq!(&map[253..], &[253, 254, 0]);

        let mut sorted = map.to_vec();

        sorted.sort_unstable();

        assert_eq!(sorted, (0..=255).collect::<Vec<u8>>());
    }

    /// An ext out of step with the hierarchy is malformed, so the writer
    /// errors instead of drawing what the state does not place.
    #[test]
    fn an_ext_out_of_step_with_its_listings_errors() {
        let file = placed_models_file();

        let mut main = from_mvox_file(&file).unwrap();

        main.ext_mut().scene_nodes.remove(&node(7));

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&file).unwrap();

        let MVoxExtNodeBody::Shape { models } =
            &mut main.ext_mut().scene_nodes.get_mut(&node(3)).unwrap().body
        else {
            panic!("node 3 is a shape");
        };

        models.push(MVoxExtShapeModel {
            object: object(1),
            frame_index: Some(1),
            extra: Vec::new(),
        });

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&file).unwrap();

        main.ext_mut().scene_nodes.get_mut(&node(7)).unwrap().id = 0;

        assert!(to_mvox_file(&main).is_err());
    }

    /// A node whose children no longer fit its entry's kind errors: a
    /// transform over two nodes, a group placing an object, a shape placing
    /// a node.
    #[test]
    fn children_that_do_not_fit_the_kind_error() {
        let file = placed_models_file();

        let mut main = from_mvox_file(&file).unwrap();

        set_children(&mut main, 0, vec![node(1), node(2)], Vec::new());

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&file).unwrap();

        set_children(
            &mut main,
            1,
            vec![node(2), node(4), node(6)],
            vec![object(0)],
        );

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&file).unwrap();

        set_children(&mut main, 3, vec![node(7)], vec![object(0)]);

        assert!(to_mvox_file(&main).is_err());
    }

    /// A transform node moved after the load errors until its frames follow.
    #[test]
    fn a_moved_transform_errors_until_its_frames_follow() {
        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        let moved = VoxHierarchyNode {
            transform: TyTransformF64::from_translation(TyVector3F64::new(2.0, 0.0, 0.0)),
            ..main.hierarchy_node(node(2)).unwrap().clone()
        };

        main.set_hierarchy_node(node(2), moved).unwrap();

        assert!(to_mvox_file(&main).is_err());

        let MVoxExtNodeBody::Transform { frames, .. } =
            &mut main.ext_mut().scene_nodes.get_mut(&node(2)).unwrap().body
        else {
            panic!("node 2 is a transform");
        };

        frames[0].translation = [2, 0, 0];

        let rebuilt = to_mvox_file(&main).unwrap();

        let MVoxSceneNodeBody::Transform(transform) = &rebuilt.scene_nodes[2].body else {
            panic!("node 2 is a transform");
        };

        assert_eq!(transform.frames[0].translation, [2, 0, 0]);
    }

    /// A second palette has no place in a MagicaVoxel file, and neither does
    /// a material past the 256 color slots.
    #[test]
    fn a_second_palette_or_a_material_past_the_slots_errors() {
        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        main.retain_palette(VoxPalette::default()).unwrap();

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        let palette_id = U32Id::<BVoxPalette>::from_u32(0);

        let value_id = main
            .palette(palette_id)
            .unwrap()
            .value_id(material(0), U32Id::from_u32(0))
            .unwrap();

        main.retain_material(palette_id, vec![value_id]).unwrap();

        assert!(to_mvox_file(&main).is_err());
    }

    /// An ext material entry for a material the palette does not hold is out
    /// of step. No entries at all is fine: no `MATL` chunks write.
    #[test]
    fn a_material_entry_the_palette_lacks_errors() {
        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        main.ext_mut()
            .materials
            .insert(material(300), Default::default());

        assert!(to_mvox_file(&main).is_err());

        let mut main = from_mvox_file(&placed_models_file()).unwrap();

        main.ext_mut().materials = BTreeMap::new();

        assert!(to_mvox_file(&main).is_ok());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use super::*;
        use crate::codec::{from_mvox_bytes, to_mvox_bytes};

        #[test]
        fn round_trips_through_mvox_bytes() {
            let file = sample_file();

            let main = from_mvox_file(&file).unwrap();

            let bytes = to_mvox_bytes(&main).unwrap();

            let reloaded = from_mvox_bytes(&bytes).unwrap();

            assert_files_eq(&to_mvox_file(&reloaded).unwrap(), &file);
        }
    }
}
