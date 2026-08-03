use crate::{
    BASE_COLOR, Error, MagicaVoxelCamera, MagicaVoxelExt, MagicaVoxelExtWrapper, MagicaVoxelFrame,
    MagicaVoxelLayer, MagicaVoxelMaterial, MagicaVoxelNode, MagicaVoxelNodeBody,
    MagicaVoxelShapeModel, MagicaVoxelUnknownChunk, Result, lin_srgba_from_srgba_u8, to_vox_value,
};
use branded_id::U32Id;
use mvox::{
    MVoxCamera, MVoxFile, MVoxFrame, MVoxLayer, MVoxMaterial, MVoxMaterialType, MVoxModel,
    MVoxSceneNode, MVoxSceneNodeBody,
};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
use ty_math::{
    TyMatrix4x4F64, TyQuaternionExt, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64,
    TyVector3I32, TyVector3U32,
};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject,
    VoxPalette, VoxValuePool,
};

/// Color indices in a MagicaVoxel palette: one material per index `0..=255`.
const PALETTE_CELLS: usize = 256;

/// MagicaVoxel's default shading token, taken by a color slot with no material.
const DEFAULT_MATERIAL_TYPE: &str = "_diffuse";

/// Reads one scalar material field, for the per-attribute value-pool build.
type ScalarField = fn(&MVoxMaterial) -> Option<f32>;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a [`VoxMain`].
///
/// Models become objects, the 256-color palette plus the `MATL` materials
/// become one shared palette of value pools, and the `nTRN` / `nGRP` / `nSHP`
/// scene graph becomes the hierarchy nodes. The state with no native voxcore
/// home, such as layers, cameras, render settings, the exact per-node frames,
/// and each material's exact optional fields, rides in a `magica-voxel` ext so
/// the file can be written back exactly.
///
/// Errors if:
///
/// 1. the geometry is malformed
/// 2. a material id is outside the palette range
/// 3. a scene-node reference dangles
/// 4. a checked insertion rejects a cross-reference
pub fn from_mvox_file(file: &MVoxFile) -> Result<VoxMain> {
    let mut state = VoxMain::default();

    let palette = build_palette(&mut state, file)?;
    let palette_id = state.add_palette(palette)?;

    for model in &file.models {
        // The model grid becomes the object's build volume directly; it may
        // carry empty margin around the live voxels.
        state.add_object(build_object(model, palette_id)?)?;
    }

    // The scene graph lands as one batch: a transform node lists its child by
    // listing position, which may lie ahead of it.
    let (nodes, roots) = build_hierarchy(file)?;
    state.add_hierarchy_nodes(nodes)?;
    state.set_root_hierarchy_node_ids(roots)?;

    let ext = MagicaVoxelExtWrapper {
        magica_voxel: magica_voxel_ext(file),
    };
    state.set_ext(Some(to_vox_value(&ext)?));

    Ok(state)
}

/// Builds the shared palette: one material per color index `0..=255`, so a
/// material's index is its color index. `baseColor` binds a color
/// value pool; with materials, `type` and the six scalar fields bind their own
/// value pools. An absent or non-finite field takes a default, since a
/// value pool holds no null and a float value pool rejects a non-finite value;
/// the exact optionals ride in the ext. Errors on a material id outside
/// `0..=255` or a duplicate id.
fn build_palette(state: &mut VoxMain, file: &MVoxFile) -> Result<VoxPalette> {
    let colors = file.resolved_palette().colors;
    let has_materials = !file.materials.is_empty();

    let mut material_by_id: HashMap<i32, &MVoxMaterial> =
        HashMap::with_capacity(file.materials.len());
    for material in &file.materials {
        if material.id < 0 || material.id as usize >= PALETTE_CELLS {
            return Err(invalid(format!(
                "material id {} is outside the palette index range 0..={}",
                material.id,
                PALETTE_CELLS - 1
            )));
        }
        if material_by_id.insert(material.id, material).is_some() {
            return Err(invalid(format!(
                "material id {} is declared more than once",
                material.id
            )));
        }
    }

    let mut palette = VoxPalette::default();

    let color_bytes: Vec<[u8; 4]> = colors.iter().map(|c| [c.r, c.g, c.b, c.a]).collect();
    let (distinct_colors, color_indices) = intern(&color_bytes, |&color| color);
    let color_value_pool_id = state.add_value_pool(
        VoxValuePool::vec_4_float(
            distinct_colors
                .iter()
                .map(|&color| <[f64; 4]>::from(lin_srgba_from_srgba_u8(TySrgbaU8::from(color))))
                .collect(),
        )
        .expect("byte-derived components are finite and the palette is non-empty"),
    );
    palette
        .add_property(
            BASE_COLOR.to_owned(),
            color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");

    // The scalars are custom MagicaVoxel attributes with no glTF bounds, so
    // their float value pools are unbounded.
    let mut attribute_indices: Vec<Vec<u32>> = Vec::new();
    if has_materials {
        const SCALARS: [(&str, ScalarField); 6] = [
            ("weight", |m| m.weight),
            ("rough", |m| m.rough),
            ("spec", |m| m.spec),
            ("ior", |m| m.ior),
            ("att", |m| m.att),
            ("flux", |m| m.flux),
        ];

        let types: Vec<String> = (0..PALETTE_CELLS)
            .map(|index| {
                material_by_id
                    .get(&(index as i32))
                    .copied()
                    .and_then(|material| material.material_type.as_ref())
                    .map(material_type_token)
                    .unwrap_or_else(|| DEFAULT_MATERIAL_TYPE.to_owned())
            })
            .collect();
        let (distinct_types, type_indices) = intern(&types, |token| token.clone());
        let type_value_pool_id = state.add_value_pool(
            VoxValuePool::string(distinct_types).expect("every palette cell yields a type token"),
        );
        palette
            .add_property("type".to_owned(), type_value_pool_id, U32Id::from_u32(0))
            .expect("the property names are distinct");
        attribute_indices.push(type_indices);

        for (name, read) in SCALARS {
            let values: Vec<f64> = (0..PALETTE_CELLS)
                .map(|index| {
                    material_by_id
                        .get(&(index as i32))
                        .copied()
                        .and_then(read)
                        // The codec accepts a non-finite scalar. NaN cannot
                        // enter a float value pool, so any non-finite value
                        // defaults. The ext keeps the exact one.
                        .filter(|value| value.is_finite())
                        .map_or(0.0, |value| value as f64)
                })
                .collect();
            let (distinct, indices) = intern(&values, |value| value.to_bits());
            let value_pool_id = state.add_value_pool(
                VoxValuePool::float(distinct)
                    .expect("the scalars are finite and every palette cell yields one"),
            );
            palette
                .add_property(name.to_owned(), value_pool_id, U32Id::from_u32(0))
                .expect("the property names are distinct");
            attribute_indices.push(indices);
        }
    }

    for index in 0..PALETTE_CELLS {
        let mut value_ids = vec![U32Id::from_u32(color_indices[index])];
        for column in &attribute_indices {
            value_ids.push(U32Id::from_u32(column[index]));
        }
        palette
            .add_material(value_ids)
            .expect("one value id per property");
    }

    Ok(palette)
}

/// Deduplicates `values` in first-seen order, returning the distinct list and
/// each input's index into it. `key` is a value's dedup key.
fn intern<T: Clone, K: Eq + Hash>(values: &[T], key: impl Fn(&T) -> K) -> (Vec<T>, Vec<u32>) {
    let mut distinct: Vec<T> = Vec::new();
    let mut index_of: HashMap<K, u32> = HashMap::new();
    let indices = values
        .iter()
        .map(|value| {
            *index_of.entry(key(value)).or_insert_with(|| {
                let index = distinct.len() as u32;
                distinct.push(value.clone());
                index
            })
        })
        .collect();
    (distinct, indices)
}

/// The `_type` token for a material shading model. Known variants map to their
/// documented tokens; an unmodeled variant keeps its stored string.
fn material_type_token(material_type: &MVoxMaterialType) -> String {
    match material_type {
        MVoxMaterialType::Diffuse => "_diffuse".to_owned(),
        MVoxMaterialType::Metal => "_metal".to_owned(),
        MVoxMaterialType::Glass => "_glass".to_owned(),
        MVoxMaterialType::Emit => "_emit".to_owned(),
        MVoxMaterialType::Other(token) => token.clone(),
    }
}

/// Builds an object from a model: a dense grid sized by the model, referencing
/// the shared palette on one layer, each voxel sampling the material at its
/// color index. Errors on an oversized grid or a voxel outside the model
/// bounds.
fn build_object(model: &MVoxModel, palette_id: U32Id<BVoxPalette>) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = model.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .map_err(|_| {
            invalid(format!(
                "model grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    // The single layer is the color index; live voxels overwrite material 0.
    object.add_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for voxel in &model.voxels {
        let position = TyVector3U32::new(voxel.x as u32, voxel.y as u32, voxel.z as u32);
        let voxel_id = object.voxel_id(position).ok_or_else(|| {
            invalid(format!(
                "model voxel ({}, {}, {}) lies outside its size [{size_x}, {size_y}, {size_z}]",
                voxel.x, voxel.y, voxel.z
            ))
        })?;
        object
            .retain_voxel(
                voxel_id,
                &[U32Id::<BVoxMaterial>::from_u32(voxel.color_index as u32)],
            )
            .expect("one sample for the one layer");
    }

    Ok(object)
}

/// Builds the hierarchy nodes, one per scene node in stored order, plus the
/// roots. A transform node places its single child, a group its children, and a
/// shape the objects for its models. The links are deduplicated to satisfy
/// voxcore's per-node uniqueness rule; the exact lists ride in the ext. Roots
/// are the nodes no other node lists as a child. Errors on a duplicate node id
/// or a dangling child reference.
fn build_hierarchy(
    file: &MVoxFile,
) -> Result<(Vec<VoxHierarchyNode>, Vec<U32Id<BVoxHierarchyNode>>)> {
    let mut position_of_id: HashMap<i32, usize> = HashMap::with_capacity(file.scene_nodes.len());
    for (position, node) in file.scene_nodes.iter().enumerate() {
        if position_of_id.insert(node.id, position).is_some() {
            return Err(invalid(format!(
                "scene node id {} is declared more than once",
                node.id
            )));
        }
    }

    let mut nodes = Vec::with_capacity(file.scene_nodes.len());
    let mut referenced = vec![false; file.scene_nodes.len()];

    for node in &file.scene_nodes {
        let name = node.attributes.name.clone().unwrap_or_default();
        let vox_node = match &node.body {
            MVoxSceneNodeBody::Transform(transform) => {
                let child_index = resolve(&position_of_id, transform.child)?;
                referenced[child_index] = true;
                VoxHierarchyNode {
                    name,
                    child_node_ids: vec![U32Id::from_u32(child_index as u32)],
                    child_object_ids: Vec::new(),
                    transform: transform_from_frames(&transform.frames),
                }
            }
            MVoxSceneNodeBody::Group(group) => {
                let mut child_node_ids = Vec::with_capacity(group.children.len());
                let mut seen = HashSet::new();
                for &child_id in &group.children {
                    let child_index = resolve(&position_of_id, child_id)?;
                    referenced[child_index] = true;
                    if seen.insert(child_index) {
                        child_node_ids.push(U32Id::from_u32(child_index as u32));
                    }
                }
                VoxHierarchyNode {
                    name,
                    child_node_ids,
                    child_object_ids: Vec::new(),
                    transform: TyTransformF64::default(),
                }
            }
            MVoxSceneNodeBody::Shape(shape) => {
                let mut child_object_ids = Vec::with_capacity(shape.models.len());
                let mut seen = HashSet::new();
                for model in &shape.models {
                    if seen.insert(model.model) {
                        child_object_ids.push(U32Id::<BVoxObject>::from_u32(model.model));
                    }
                }
                VoxHierarchyNode {
                    name,
                    child_node_ids: Vec::new(),
                    child_object_ids,
                    transform: TyTransformF64::default(),
                }
            }
        };
        nodes.push(vox_node);
    }

    let roots = (0..nodes.len())
        .filter(|&position| !referenced[position])
        .map(|position| U32Id::from_u32(position as u32))
        .collect();

    Ok((nodes, roots))
}

/// The position of the scene node with id `id`, or an error if none has it.
fn resolve(position_of_id: &HashMap<i32, usize>, id: i32) -> Result<usize> {
    position_of_id.get(&id).copied().ok_or_else(|| {
        invalid(format!(
            "scene node references node {id}, which does not exist"
        ))
    })
}

/// The transform projecting a transform node's first frame, or the identity
/// when it has none. The exact frames ride in the ext, so this is only a usable
/// approximation.
fn transform_from_frames(frames: &[MVoxFrame]) -> TyTransformF64 {
    match frames.first() {
        Some(frame) => transform_from_frame(frame),
        None => TyTransformF64::default(),
    }
}

/// Projects one keyframe to a [`TyTransformF64`]. The rotation is the frame's
/// signed-permutation matrix; an improper one (a mirror) is split into a proper
/// rotation and a negative x scale so voxcore's unit-quaternion invariant
/// holds.
fn transform_from_frame(frame: &MVoxFrame) -> TyTransformF64 {
    let position = TyVector3I32::from_array(frame.translation).as_dvec3();

    let signed = frame.rotation.to_matrix();
    let mut matrix = [[0.0f64; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            matrix[row][column] = signed[row][column] as f64;
        }
    }

    let mut scale = TyVector3F64::new(1.0, 1.0, 1.0);
    if determinant(&matrix) < 0.0 {
        // M = R * diag(-1, 1, 1), so negating column 0 leaves a proper
        // rotation.
        for row in &mut matrix {
            row[0] = -row[0];
        }
        scale.x = -1.0;
    }

    // Read the proper rotation into a column-major matrix and decode it. The
    // frame is a signed permutation, so after the mirror split it is always a
    // proper rotation.
    let rotation = TyMatrix4x4F64::from_cols_array_2d(&[
        [matrix[0][0], matrix[1][0], matrix[2][0], 0.0],
        [matrix[0][1], matrix[1][1], matrix[2][1], 0.0],
        [matrix[0][2], matrix[1][2], matrix[2][2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let rotation = TyQuaternionF64::from_rotation_matrix(rotation)
        .expect("the frame is a proper rotation after the mirror split");

    TyTransformF64::new(position, rotation, scale)
}

/// The determinant of a 3x3 matrix, the scalar triple product of its columns.
fn determinant(matrix: &[[f64; 3]; 3]) -> f64 {
    let column = |c: usize| TyVector3F64::new(matrix[0][c], matrix[1][c], matrix[2][c]);
    column(0).dot(column(1).cross(column(2)))
}

/// Builds the `magica-voxel` ext payload from the state with no native home.
fn magica_voxel_ext(file: &MVoxFile) -> MagicaVoxelExt {
    MagicaVoxelExt {
        version: file.version,
        palette_present: file.palette.is_some(),
        materials: file
            .materials
            .iter()
            .map(|material| MagicaVoxelMaterial {
                id: material.id,
                material_type: material.material_type.as_ref().map(material_type_token),
                weight: material.weight,
                rough: material.rough,
                spec: material.spec,
                ior: material.ior,
                att: material.att,
                flux: material.flux,
                extra: material.extra.0.clone(),
            })
            .collect(),
        scene_nodes: file.scene_nodes.iter().map(node_provenance).collect(),
        layers: file.layers.iter().map(layer_provenance).collect(),
        render_objects: file
            .render_objects
            .iter()
            .map(|render_object| render_object.attributes.0.clone())
            .collect(),
        cameras: file.cameras.iter().map(camera_provenance).collect(),
        palette_notes: file.palette_notes.clone(),
        index_map: file.index_map.map(|map| map.to_vec()),
        unknown_chunks: file
            .unknown_chunks
            .iter()
            .map(|chunk| MagicaVoxelUnknownChunk {
                id: chunk.id,
                content: chunk.content.clone(),
                children: chunk.children.clone(),
            })
            .collect(),
    }
}

/// The ext provenance for one scene node: its id, attributes, and per-kind
/// body.
fn node_provenance(node: &MVoxSceneNode) -> MagicaVoxelNode {
    MagicaVoxelNode {
        id: node.id,
        name: node.attributes.name.clone(),
        hidden: node.attributes.hidden,
        attr_extra: node.attributes.extra.0.clone(),
        body: match &node.body {
            MVoxSceneNodeBody::Transform(transform) => MagicaVoxelNodeBody::Transform {
                child: transform.child,
                layer: transform.layer,
                frames: transform.frames.iter().map(frame_provenance).collect(),
            },
            MVoxSceneNodeBody::Group(group) => MagicaVoxelNodeBody::Group {
                children: group.children.clone(),
            },
            MVoxSceneNodeBody::Shape(shape) => MagicaVoxelNodeBody::Shape {
                models: shape
                    .models
                    .iter()
                    .map(|model| MagicaVoxelShapeModel {
                        model: model.model,
                        frame_index: model.frame_index,
                        extra: model.extra.0.clone(),
                    })
                    .collect(),
            },
        },
    }
}

/// The ext provenance for one transform-node frame.
fn frame_provenance(frame: &MVoxFrame) -> MagicaVoxelFrame {
    MagicaVoxelFrame {
        rotation: frame.rotation.0,
        translation: frame.translation,
        frame_index: frame.frame_index,
        extra: frame.extra.0.clone(),
    }
}

/// The ext provenance for one layer.
fn layer_provenance(layer: &MVoxLayer) -> MagicaVoxelLayer {
    MagicaVoxelLayer {
        id: layer.id,
        name: layer.name.clone(),
        hidden: layer.hidden,
        extra: layer.extra.0.clone(),
    }
}

/// The ext provenance for one camera.
fn camera_provenance(camera: &MVoxCamera) -> MagicaVoxelCamera {
    MagicaVoxelCamera {
        id: camera.id,
        mode: camera.mode.clone(),
        focus: camera.focus,
        angle: camera.angle,
        radius: camera.radius,
        frustum: camera.frustum,
        fov: camera.fov,
        extra: camera.extra.0.clone(),
    }
}

/// Invalid-data error from a message.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

#[cfg(test)]
mod tests {
    use crate::{
        BASE_COLOR, from_mvox_bytes, from_mvox_file, lin_srgba_from_srgba_u8, to_mvox_bytes,
        to_mvox_file,
    };
    use branded_id::U32Id;
    use mvox::{
        MVoxCamera, MVoxColor, MVoxDict, MVoxFile, MVoxFrame, MVoxGroupNode, MVoxLayer,
        MVoxMaterial, MVoxMaterialType, MVoxModel, MVoxNodeAttributes, MVoxPalette,
        MVoxRenderObject, MVoxRotation, MVoxSceneNode, MVoxSceneNodeBody, MVoxShapeModel,
        MVoxShapeNode, MVoxTransformNode, MVoxUnknownChunk, MVoxVoxel,
    };
    use std::{array, collections::BTreeSet};
    use ty_math::{
        TyHexColor, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32,
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxMap, VoxObject,
        VoxPalette, VoxValue, VoxValuePool,
    };

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_owned(), value.to_owned())
    }

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        lin_srgba_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color")).into()
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

    /// A resolved voxel: `x`, `y`, `z`, and an `rgba` color.
    type ResolvedVoxel = (u8, u8, u8, (u8, u8, u8, u8));

    /// A model's voxels as `(x, y, z, (r, g, b, a))`, order-independent so a
    /// synthesized model compares without depending on raster order.
    fn resolved_voxels(model: &MVoxModel, palette: &MVoxPalette) -> BTreeSet<ResolvedVoxel> {
        model
            .voxels
            .iter()
            .map(|voxel| {
                let color = palette.colors[voxel.color_index as usize];
                (
                    voxel.x,
                    voxel.y,
                    voxel.z,
                    (color.r, color.g, color.b, color.a),
                )
            })
            .collect()
    }

    /// A state carrying no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `rgba` palette, placed by a small
    /// hierarchy of a nested group and two roots. The writer must synthesize a
    /// MagicaVoxel file from this rather than read an ext.
    fn source_state() -> VoxMain {
        let mut state = VoxMain::default();

        // One baseColor palette: a transparent placeholder, then red,
        // green, blue.
        let value_pool_id = state.add_value_pool(
            VoxValuePool::vec_4_float(
                ["#00000000", "#FF0000FF", "#00FF00FF", "#0000FFFF"]
                    .iter()
                    .map(|hex| linear_rgba(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        for index in 0..4 {
            palette
                .add_material(vec![U32Id::from_u32(index)])
                .expect("one value-index for the one binding");
        }
        let palette_id = state.add_palette(palette).unwrap();
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.add_layer(palette_id, material_id(0));
        for (x, material_index) in [(0u32, 1u32), (1, 2)] {
            let voxel_id = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
        }
        state.add_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.add_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(3)])
            .expect("one sample for the one layer");
        state.add_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::ONE,
            )
        };

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state
            .add_hierarchy_nodes(vec![
                VoxHierarchyNode {
                    name: "group".to_owned(),
                    child_node_ids: vec![node_id(1)],
                    child_object_ids: Vec::new(),
                    transform: TyTransformF64::default(),
                },
                VoxHierarchyNode {
                    name: "wide".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(0)],
                    transform: placed_at(5.0, 0.0, 0.0),
                },
                VoxHierarchyNode {
                    name: "unit".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(1)],
                    transform: placed_at(0.0, 3.0, 0.0),
                },
            ])
            .unwrap();
        state
            .set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        state.validate().expect("a well-formed source state");
        state
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
    fn round_trips_through_mvox_bytes() {
        let file = sample_file();
        let state = from_mvox_file(&file).unwrap();
        let bytes = to_mvox_bytes(&state).unwrap();
        let reloaded = from_mvox_bytes(&bytes).unwrap();
        assert_files_eq(&to_mvox_file(&reloaded).unwrap(), &file);
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

    /// A default state has no objects and no ext, so the writer synthesizes an
    /// empty file rather than erroring on the missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxMain::default();
        let file = to_mvox_file(&state).unwrap();
        assert!(file.models.is_empty());
    }

    /// A state with no `magica-voxel` ext, such as one cross-loaded from
    /// another format, synthesizes a file: one model per object, a global
    /// palette gathering every used color, and a scene graph the decoder reads
    /// back.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let state = source_state();
        let file = to_mvox_file(&state).unwrap();
        let palette = file.palette.as_ref().expect("synthesis writes a palette");

        let red = (0xFF, 0, 0, 0xFF);
        let green = (0, 0xFF, 0, 0xFF);
        let blue = (0, 0, 0xFF, 0xFF);

        assert_eq!(file.models.len(), 2);
        assert_eq!(file.models[0].size, [2, 1, 1]);
        assert_eq!(
            resolved_voxels(&file.models[0], palette),
            BTreeSet::from([(0, 0, 0, red), (1, 0, 0, green)])
        );
        assert_eq!(file.models[1].size, [1, 1, 1]);
        assert_eq!(
            resolved_voxels(&file.models[1], palette),
            BTreeSet::from([(0, 0, 0, blue)])
        );

        // The synthesized palette and scene graph read back into a valid state.
        let reloaded = from_mvox_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// A foreign ext, here `voxel-max`, is ignored by the MagicaVoxel writer:
    /// it synthesizes from the scene rather than failing to parse the wrong
    /// ext.
    #[test]
    fn synthesizes_past_a_foreign_ext() {
        let mut state = source_state();
        state.set_ext(Some(VoxValue::Object(VoxMap(vec![(
            "voxel-max".to_owned(),
            VoxValue::Null,
        )]))));
        let file = to_mvox_file(&state).unwrap();
        assert_eq!(file.models.len(), 2);
    }

    #[test]
    fn rejects_a_material_id_outside_the_palette_range() {
        let file = MVoxFile {
            materials: vec![MVoxMaterial {
                id: 300,
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
    }

    /// The codec accepts a non-finite scalar, but a Float value pool must be
    /// finite. from_mvox_file defaults the value-pool copy and the ext keeps
    /// the exact value, so the material still round-trips.
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
    }

    #[test]
    fn rejects_a_dangling_scene_node_child() {
        let file = MVoxFile {
            scene_nodes: vec![MVoxSceneNode {
                id: 0,
                attributes: MVoxNodeAttributes::default(),
                body: MVoxSceneNodeBody::Transform(MVoxTransformNode {
                    child: 99,
                    layer: -1,
                    frames: vec![MVoxFrame::default()],
                }),
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
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

    #[test]
    fn rejects_a_voxel_outside_its_model() {
        let file = MVoxFile {
            models: vec![MVoxModel {
                size: [1, 1, 1],
                voxels: vec![MVoxVoxel {
                    x: 5,
                    y: 0,
                    z: 0,
                    color_index: 1,
                }],
            }],
            ..Default::default()
        };
        assert!(from_mvox_file(&file).is_err());
    }
}
