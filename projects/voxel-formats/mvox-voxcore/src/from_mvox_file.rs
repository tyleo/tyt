use crate::{
    Error, MVoxVoxMain, Result, material_type_token, mvox_ext_from_file, transform_from_frames,
};
use branded_id::U32Id;
use mvox::{MVoxFile, MVoxMaterial, MVoxModel, MVoxSceneNodeBody};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject,
    VoxPalette, VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
};

/// Color indices in a MagicaVoxel palette: one material per index `0..=255`.
const PALETTE_CELLS: usize = 256;

/// MagicaVoxel's default shading token, taken by a color slot with no material.
const DEFAULT_MATERIAL_TYPE: &str = "_diffuse";

/// Reads one scalar material field, for the per-attribute value-pool build.
type ScalarField = fn(&MVoxMaterial) -> Option<f32>;

/// Loads a decoded MagicaVoxel [`MVoxFile`] into a [`MVoxVoxMain`], the
/// inverse of [`to_mvox_file`](crate::to_mvox_file). Models become objects,
/// the 256-color palette plus the `MATL` materials become one shared palette
/// of value pools, and the `nTRN` / `nGRP` / `nSHP` scene graph becomes the
/// hierarchy nodes, one per scene node. The rest of the MagicaVoxel state,
/// such as layers, cameras, render settings, the exact per-node frames, and
/// each material's exact optional fields, becomes the ext.
///
/// Errors if:
///
/// 1. the geometry is malformed
/// 2. a material id is outside the palette range
/// 3. a scene-node reference dangles
/// 4. a checked insertion rejects a cross-reference
pub fn from_mvox_file(file: &MVoxFile) -> Result<MVoxVoxMain> {
    let ext = mvox_ext_from_file(file)?;
    let mut main = VoxMain::default();

    let palette = build_palette(&mut main, file)?;
    let palette_id = main.retain_palette(palette)?;

    for model in &file.models {
        // The model grid becomes the object's build volume directly; it may
        // carry empty margin around the live voxels.
        main.retain_object(build_object(model, palette_id)?)?;
    }

    // The scene graph lands as one batch: a transform node lists its child by
    // listing position, which may lie ahead of it.
    let (nodes, roots) = build_hierarchy(file)?;
    main.retain_hierarchy_nodes(nodes)?;
    main.set_root_hierarchy_node_ids(roots)?;

    Ok(main.put_ext(ext))
}

/// Builds the shared palette: one material per color index `0..=255`, so a
/// material's index is its color index. `baseColor` binds a color
/// value pool; with materials, `type` and the six scalar fields bind their own
/// value pools. An absent field or a NaN takes a default, since a value pool
/// holds no null and a float value pool rejects NaN; the infinities carry
/// across, and the exact optionals ride in the ext. Errors on a material id
/// outside `0..=255` or a duplicate id.
fn build_palette(main: &mut VoxMain<()>, file: &MVoxFile) -> Result<VoxPalette> {
    let colors = file.resolved_palette().colors;
    let has_materials = !file.materials.is_empty();

    let mut material_by_id: HashMap<i32, &MVoxMaterial> =
        HashMap::with_capacity(file.materials.len());
    for material in &file.materials {
        if material.id < 0 || material.id as usize >= PALETTE_CELLS {
            return Err(Error::invalid(format!(
                "material id {} is outside the palette index range 0..={}",
                material.id,
                PALETTE_CELLS - 1
            )));
        }
        if material_by_id.insert(material.id, material).is_some() {
            return Err(Error::invalid(format!(
                "material id {} is declared more than once",
                material.id
            )));
        }
    }

    let mut palette = VoxPalette::default();

    let color_bytes: Vec<[u8; 4]> = colors.iter().map(|c| [c.r, c.g, c.b, c.a]).collect();
    let (distinct_colors, color_indices) = intern(&color_bytes, |&color| color);
    let color_value_pool_id = main.retain_value_pool(
        VoxValuePool::vec_4_float(
            distinct_colors
                .iter()
                .map(|&color| <[f64; 4]>::from(lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(color))))
                .collect(),
        )
        .expect("byte-derived components are finite and the palette is non-empty"),
    );
    palette
        .retain_property(
            BASE_COLOR.to_owned(),
            color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");

    // The scalars are custom MagicaVoxel attributes, so the glTF vocabulary
    // range check does not reach them.
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
        let type_value_pool_id = main.retain_value_pool(VoxValuePool::string(distinct_types));
        palette
            .retain_property("type".to_owned(), type_value_pool_id, U32Id::from_u32(0))
            .expect("the property names are distinct");
        attribute_indices.push(type_indices);

        for (name, read) in SCALARS {
            let values: Vec<f64> = (0..PALETTE_CELLS)
                .map(|index| {
                    material_by_id
                        .get(&(index as i32))
                        .copied()
                        .and_then(read)
                        // The codec accepts a NaN, which no float value pool
                        // holds, so it defaults. The infinities the wire
                        // spells carry across.
                        .filter(|value| !value.is_nan())
                        .map_or(0.0, |value| value as f64)
                })
                .collect();
            let (distinct, indices) = intern(&values, |value| value.to_bits());
            let value_pool_id = main.retain_value_pool(
                VoxValuePool::float(distinct)
                    .expect("the scalars are finite and every palette cell yields one"),
            );
            palette
                .retain_property(name.to_owned(), value_pool_id, U32Id::from_u32(0))
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
            .retain_material(value_ids)
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

/// Builds an object from a model: a dense grid sized by the model, referencing
/// the shared palette on one layer, each voxel sampling the material at its
/// color index. Errors on an oversized grid or a voxel outside the model
/// bounds.
fn build_object(model: &MVoxModel, palette_id: U32Id<BVoxPalette>) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = model.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .map_err(|_| {
            Error::invalid(format!(
                "model grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    // The single layer is the color index; live voxels overwrite material 0.
    object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for voxel in &model.voxels {
        let position = TyVector3U32::new(voxel.x as u32, voxel.y as u32, voxel.z as u32);
        let voxel_id = object.voxel_id(position).ok_or_else(|| {
            Error::invalid(format!(
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
            return Err(Error::invalid(format!(
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
        Error::invalid(format!(
            "scene node references node {id}, which does not exist"
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::from_mvox_file;
    use mvox::{
        MVoxFile, MVoxFrame, MVoxMaterial, MVoxModel, MVoxNodeAttributes, MVoxSceneNode,
        MVoxSceneNodeBody, MVoxTransformNode, MVoxVoxel,
    };

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
