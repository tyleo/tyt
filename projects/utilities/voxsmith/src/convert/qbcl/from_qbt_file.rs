use crate::{
    BASE_COLOR_FACTOR, Error, QubicleQbtExt, QubicleQbtExtWrapper, QubicleQbtNode, Result,
    to_vox_value,
};
use branded_id::U32Id;
use qbcl::qbt::{QbtFile, QbtMatrix, QbtNode};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool,
};

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a [`VoxMain`].
///
/// Matrix and compound grids become objects sharing one `baseColorFactor`
/// palette, and the scene tree becomes the hierarchy nodes. The state with no
/// native voxcore home, such as the per-voxel visibility masks, matrix names,
/// placements, scales, and pivots, the color map, the global scale, the
/// version, and any unknown nodes, rides in a `qubicle-qbt` ext so the file can
/// be written back exactly.
///
/// Errors on a matrix grid that exceeds the dense limit, or if
/// [`VoxMain::validate`](voxcore::VoxMain::validate) rejects the result.
pub fn from_qbt_file(file: &QbtFile) -> Result<VoxMain> {
    let mut state = VoxMain::default();

    let (palette, materials) = build_palette(&mut state, &file.root);
    let palette_id = state.add_palette(palette);

    let mut nodes = Vec::new();
    let root_id = build_node(&file.root, &mut state, palette_id, &materials, &mut nodes)?;
    state.set_root_hierarchy_nodes(vec![root_id]);

    let ext = QubicleQbtExtWrapper {
        qubicle_qbt: QubicleQbtExt {
            version: file.version,
            global_scale: file.global_scale,
            color_map: file
                .color_map
                .iter()
                .map(|color| [color.r, color.g, color.b, color.a])
                .collect(),
            nodes,
        },
    };
    state.set_ext(Some(to_vox_value(&ext)?));

    state.validate()?;
    Ok(state)
}

/// Builds the hierarchy node for one scene node and its subtree, adding it and
/// any objects to the state and appending its provenance to `nodes` so the ext
/// entry at each index lines up with the hierarchy node id. Returns the new
/// node's id.
fn build_node(
    node: &QbtNode,
    state: &mut VoxMain,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
    nodes: &mut Vec<QubicleQbtNode>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let id = match node {
        QbtNode::Matrix(matrix) => {
            // The matrix grid becomes the object's build volume directly; it
            // may carry empty margin. The masks are read from that same grid.
            let object = build_object(matrix, palette, materials)?;
            let masks = masks_of(&object, matrix);
            let object_id = state.add_object(object);
            let hierarchy = VoxHierarchyNode {
                name: matrix.name.clone(),
                child_nodes: Vec::new(),
                child_objects: vec![object_id],
                transform: translation(matrix.position),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(QubicleQbtNode::Matrix {
                name: matrix.name.clone(),
                position: matrix.position,
                local_scale: matrix.local_scale,
                pivot: matrix.pivot,
                masks,
            });
            id
        }
        QbtNode::Model(model) => {
            let mut child_nodes = Vec::with_capacity(model.children.len());
            for child in &model.children {
                child_nodes.push(build_node(child, state, palette, materials, nodes)?);
            }
            let hierarchy = VoxHierarchyNode {
                name: String::new(),
                child_nodes,
                child_objects: Vec::new(),
                transform: TyTransformF64::default(),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(QubicleQbtNode::Model);
            id
        }
        QbtNode::Compound(compound) => {
            let mut child_nodes = Vec::with_capacity(compound.children.len());
            for child in &compound.children {
                child_nodes.push(build_node(child, state, palette, materials, nodes)?);
            }
            // The compound grid becomes the object's build volume directly; it
            // may carry empty margin. The masks are read from that same grid.
            let object = build_object(&compound.matrix, palette, materials)?;
            let masks = masks_of(&object, &compound.matrix);
            let object_id = state.add_object(object);
            let hierarchy = VoxHierarchyNode {
                name: compound.matrix.name.clone(),
                child_nodes,
                child_objects: vec![object_id],
                transform: translation(compound.matrix.position),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(QubicleQbtNode::Compound {
                name: compound.matrix.name.clone(),
                position: compound.matrix.position,
                local_scale: compound.matrix.local_scale,
                pivot: compound.matrix.pivot,
                masks,
            });
            id
        }
        QbtNode::Unknown(unknown) => {
            let hierarchy = VoxHierarchyNode {
                name: String::new(),
                child_nodes: Vec::new(),
                child_objects: Vec::new(),
                transform: TyTransformF64::default(),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(QubicleQbtNode::Unknown {
                type_id: unknown.type_id,
                data: unknown.data.clone(),
            });
            id
        }
    };
    Ok(id)
}

/// Builds the one shared palette: a color pool of one entry per distinct color
/// across every matrix and compound voxel in the tree, bound to
/// `baseColorFactor`, with one material per color and a map from a color to its
/// material. The pool is added to `state`. A tree with no solid voxels gets a
/// single placeholder color so objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain,
    root: &QbtNode,
) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 3]> = Vec::new();
    let mut seen: HashSet<[u8; 3]> = HashSet::new();
    collect_colors(root, &mut order, &mut seen);
    if order.is_empty() {
        order.push([0, 0, 0]);
    }

    // A Qubicle voxel carries no alpha, so colors ride in a shared sRGB pool as
    // float components in `[0, 1]`; each material draws one value id into it.
    let pool = state.add_value_pool(
        VoxValuePool::srgb(order.iter().map(|&color| color_floats(color)).collect())
            .expect("byte-derived components are in range and the list is non-empty"),
    );

    let mut palette = VoxPalette::default();
    palette
        .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
        .expect("the property names are distinct");
    let mut materials = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material = palette
            .add_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        materials.insert(*color, material);
    }

    (palette, materials)
}

/// Collects the distinct colors of a node and its subtree, in first-seen order.
fn collect_colors(node: &QbtNode, order: &mut Vec<[u8; 3]>, seen: &mut HashSet<[u8; 3]>) {
    match node {
        QbtNode::Matrix(matrix) => collect_matrix(matrix, order, seen),
        QbtNode::Model(model) => {
            for child in &model.children {
                collect_colors(child, order, seen);
            }
        }
        QbtNode::Compound(compound) => {
            collect_matrix(&compound.matrix, order, seen);
            for child in &compound.children {
                collect_colors(child, order, seen);
            }
        }
        QbtNode::Unknown(_) => {}
    }
}

/// Collects the distinct colors of one matrix's solid voxels.
fn collect_matrix(matrix: &QbtMatrix, order: &mut Vec<[u8; 3]>, seen: &mut HashSet<[u8; 3]>) {
    for voxel in &matrix.voxels {
        if voxel.is_empty() {
            continue;
        }
        let color = [voxel.r, voxel.g, voxel.b];
        if seen.insert(color) {
            order.push(color);
        }
    }
}

/// Builds an object from a matrix: a dense grid sized by the matrix,
/// referencing the shared palette on one layer, each solid voxel sampling its
/// color material. Errors on an oversized grid.
fn build_object(
    matrix: &QbtMatrix,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .ok_or_else(|| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.add_layer(palette, U32Id::<BVoxMaterial>::from_u32(0));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_z {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let material = materials
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(id, &[material])
                    .expect("one sample for the one layer");
            }
        }
    }

    Ok(object)
}

/// The visibility masks of an object's solid voxels, in live-voxel raster
/// order, read back from the matrix the object was built from.
fn masks_of(object: &VoxObject, matrix: &QbtMatrix) -> Vec<u8> {
    object
        .iter_live()
        .map(|id| {
            let position = object
                .voxel_position(id)
                .expect("a live voxel is within the grid");
            matrix
                .voxel(position.x, position.y, position.z)
                .map_or(0, |voxel| voxel.mask)
        })
        .collect()
}

/// A translation-only transform from a scene position.
fn translation(position: [i32; 3]) -> TyTransformF64 {
    TyTransformF64::from_translation(TyVector3I32::from_array(position).as_dvec3())
}

/// The float sRGB components in `[0, 1]` of an `[r, g, b]` byte color.
fn color_floats(color: [u8; 3]) -> [f64; 3] {
    TySrgbU8::from(color).into_format::<f64>().into()
}

#[cfg(test)]
mod tests {
    use crate::{from_qbt_bytes, from_qbt_file, to_qbt_bytes, to_qbt_file};
    use qbcl::qbt::{
        QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
    };
    use voxcore::VoxMain;

    /// A matrix node with two solid voxels in a `[2, 1, 1]` grid.
    fn matrix_node() -> QbtNode {
        QbtNode::Matrix(QbtMatrix {
            name: "matrix".to_owned(),
            position: [1, 2, 3],
            local_scale: [1, 1, 1],
            pivot: [0.5, 0.0, 0.0],
            size: [2, 1, 1],
            voxels: vec![
                QbtVoxel::new(10, 20, 30, 0x7e),
                QbtVoxel::new(1, 2, 3, 0x01),
            ],
        })
    }

    /// A compound node carrying one baked voxel and one empty model child.
    fn compound_node() -> QbtNode {
        QbtNode::Compound(QbtCompound {
            matrix: QbtMatrix {
                name: "compound".to_owned(),
                position: [-1, -2, -3],
                local_scale: [2, 2, 2],
                pivot: [0.0, 0.0, 0.0],
                size: [1, 1, 1],
                voxels: vec![QbtVoxel::new(40, 50, 60, 0xff)],
            },
            children: vec![QbtNode::Model(QbtModel::default())],
        })
    }

    /// A file exercising a model root grouping a matrix, a compound, and an
    /// unknown node, with a color map and a global scale.
    fn sample_file() -> QbtFile {
        QbtFile {
            version: (1, 0),
            global_scale: [1.0, 2.0, 0.5],
            color_map: vec![QbtColor::new(10, 20, 30, 255), QbtColor::new(1, 2, 3, 255)],
            root: QbtNode::Model(QbtModel {
                children: vec![
                    matrix_node(),
                    compound_node(),
                    QbtNode::Unknown(QbtUnknownNode {
                        type_id: 99,
                        data: vec![9, 8, 7],
                    }),
                ],
            }),
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_through_qbt_bytes() {
        let file = sample_file();
        let state = from_qbt_file(&file).unwrap();
        let bytes = to_qbt_bytes(&state).unwrap();
        let reloaded = from_qbt_bytes(&bytes).unwrap();
        assert_eq!(to_qbt_file(&reloaded).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbtFile::default();
        let state = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&state).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbtFile {
            root: matrix_node(),
            ..Default::default()
        };
        let state = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&state).unwrap(), file);
    }

    #[test]
    fn errors_without_qubicle_qbt_ext() {
        let state = VoxMain::default();
        assert!(to_qbt_file(&state).is_err());
    }

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbtFile {
            root: QbtNode::Matrix(QbtMatrix {
                size: [2048, 2048, 2048],
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(from_qbt_file(&file).is_err());
    }
}
