use crate::{Error, QbtExtNode, QbtVoxMain, Result, qbt_ext_from_file};
use branded_id::U32Id;
use qbcl::qbt::{QbtFile, QbtMatrix, QbtNode};
use std::collections::{BTreeMap, HashMap, HashSet};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
};

/// Loads a decoded Qubicle Binary Tree [`QbtFile`] into a [`QbtVoxMain`],
/// the inverse of [`to_qbt_file`](crate::to_qbt_file). Matrix and compound
/// grids become objects sharing one `baseColor` palette, and the scene tree
/// becomes the hierarchy nodes, each named for its scene node at its
/// position. The rest of the Qubicle state goes to the ext, the per-node
/// part keyed by node id. The per-voxel visibility masks are not kept: the
/// writer derives them from the grid.
///
/// Errors on a matrix grid that exceeds the dense limit, or on a
/// cross-reference the checked insertions reject.
pub fn from_qbt_file(file: &QbtFile) -> Result<QbtVoxMain> {
    let mut main = VoxMain::default();
    let mut nodes = BTreeMap::new();

    let (palette, material_ids) = build_palette(&mut main, &file.root);
    let palette_id = main.retain_palette(palette)?;

    let root_id = build_node(&file.root, &mut main, palette_id, &material_ids, &mut nodes)?;
    main.set_root_hierarchy_node_ids(vec![root_id])?;

    Ok(main.put_ext(qbt_ext_from_file(file, nodes)))
}

/// Builds the hierarchy node for one scene node and its subtree, adding it
/// and any objects to `main` and its provenance to `nodes` under the new
/// node's id. Returns that id.
fn build_node(
    node: &QbtNode,
    main: &mut VoxMain<()>,
    palette_id: U32Id<BVoxPalette>,
    material_ids: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
    nodes: &mut BTreeMap<U32Id<BVoxHierarchyNode>, QbtExtNode>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let (node_id, entry) = match node {
        QbtNode::Matrix(matrix) => {
            // The matrix grid becomes the object's build volume directly. It
            // may carry empty margin.
            let object = build_object(matrix, palette_id, material_ids)?;
            let object_id = main.retain_object(object)?;
            let hierarchy = VoxHierarchyNode {
                name: matrix.name.clone(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
                transform: translation(matrix.position),
            };
            let node_id = main.retain_hierarchy_node(hierarchy)?;
            let entry = QbtExtNode::Matrix {
                local_scale: matrix.local_scale,
                pivot: matrix.pivot,
            };
            (node_id, entry)
        }
        QbtNode::Model(model) => {
            let mut child_node_ids = Vec::with_capacity(model.children.len());
            for child in &model.children {
                child_node_ids.push(build_node(child, main, palette_id, material_ids, nodes)?);
            }
            let hierarchy = VoxHierarchyNode {
                name: String::new(),
                child_node_ids,
                child_object_ids: Vec::new(),
                transform: TyTransformF64::default(),
            };
            let node_id = main.retain_hierarchy_node(hierarchy)?;
            (node_id, QbtExtNode::Model)
        }
        QbtNode::Compound(compound) => {
            let mut child_node_ids = Vec::with_capacity(compound.children.len());
            for child in &compound.children {
                child_node_ids.push(build_node(child, main, palette_id, material_ids, nodes)?);
            }
            // The compound grid becomes the object's build volume directly.
            // It may carry empty margin.
            let object = build_object(&compound.matrix, palette_id, material_ids)?;
            let object_id = main.retain_object(object)?;
            let hierarchy = VoxHierarchyNode {
                name: compound.matrix.name.clone(),
                child_node_ids,
                child_object_ids: vec![object_id],
                transform: translation(compound.matrix.position),
            };
            let node_id = main.retain_hierarchy_node(hierarchy)?;
            let entry = QbtExtNode::Compound {
                local_scale: compound.matrix.local_scale,
                pivot: compound.matrix.pivot,
            };
            (node_id, entry)
        }
        QbtNode::Unknown(unknown) => {
            let hierarchy = VoxHierarchyNode {
                name: String::new(),
                child_node_ids: Vec::new(),
                child_object_ids: Vec::new(),
                transform: TyTransformF64::default(),
            };
            let node_id = main.retain_hierarchy_node(hierarchy)?;
            let entry = QbtExtNode::Unknown {
                type_id: unknown.type_id,
                data: unknown.data.clone(),
            };
            (node_id, entry)
        }
    };
    nodes.insert(node_id, entry);
    Ok(node_id)
}

/// Builds the one shared palette: a color value pool of one entry per distinct
/// color across every matrix and compound voxel in the tree, bound to
/// `baseColor`, with one material per color and a map from a color to its
/// material. The value pool is added to `main`. A tree with no solid voxels
/// gets a single placeholder color so objects have a default material to
/// sample.
fn build_palette(
    main: &mut VoxMain<()>,
    root: &QbtNode,
) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 3]> = Vec::new();
    let mut seen: HashSet<[u8; 3]> = HashSet::new();
    collect_colors(root, &mut order, &mut seen);
    if order.is_empty() {
        order.push([0, 0, 0]);
    }

    // A Qubicle voxel carries no alpha, so colors decode to linear light and
    // ride in a shared `vec-3-float` value pool. Each material draws one value
    // id into it.
    let value_pool_id = main.retain_value_pool(
        VoxValuePool::vec_3_float(order.iter().map(|&color| color_floats(color)).collect())
            .expect("byte-derived components are finite and the list is non-empty"),
    );

    let mut palette = VoxPalette::default();
    palette
        .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
        .expect("the property names are distinct");
    let mut material_ids = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material_id = palette
            .retain_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        material_ids.insert(*color, material_id);
    }

    (palette, material_ids)
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
    palette_id: U32Id<BVoxPalette>,
    material_ids: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .map_err(|_| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_z {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let material_id = material_ids
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let voxel_id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(voxel_id, &[material_id])
                    .expect("one sample for the one layer");
            }
        }
    }

    Ok(object)
}

/// A translation-only transform from a scene position.
fn translation(position: [i32; 3]) -> TyTransformF64 {
    TyTransformF64::from_translation(TyVector3I32::from_array(position).as_dvec3())
}

/// The linear-light components of an `[r, g, b]` byte color.
fn color_floats(color: [u8; 3]) -> [f64; 3] {
    let [red, green, blue] = color;
    let linear = lin_srgba_f64_from_srgba_u8(TySrgbaU8::new(red, green, blue, 255));
    [linear.red, linear.green, linear.blue]
}

#[cfg(test)]
mod tests {
    use crate::from_qbt_file;
    use qbcl::qbt::{QbtFile, QbtMatrix, QbtNode};

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
