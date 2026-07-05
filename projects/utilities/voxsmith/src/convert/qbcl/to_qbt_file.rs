use crate::{
    Error, QubicleQbtExtWrapper, QubicleQbtNode, Result, cell_color, from_vox_value,
    object_color_ref,
};
use branded_id::U32Id;
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode, VoxMain, VoxObject};

/// Writes a [`VoxMain`] back to a decoded Qubicle Binary Tree [`QbtFile`], the
/// inverse of [`from_qbt_file`](crate::from_qbt_file).
///
/// Requires the `qubicle-qbt` ext the forward path writes; without it the file
/// cannot be rebuilt. The scene tree is walked from the single root, each
/// matrix or compound object emitting its grid with the visibility masks and
/// color from the ext and the palette.
///
/// Errors if the ext is missing, its node entries do not line up with the
/// hierarchy, the state does not have exactly one root, or a mask list does not
/// match its object.
pub fn to_qbt_file(state: &VoxMain) -> Result<QbtFile> {
    let ext = match state.ext() {
        Some(ext) => from_vox_value::<QubicleQbtExtWrapper>(ext)?.qubicle_qbt,
        None => {
            return Err(Error::invalid(
                "state has no qubicle-qbt ext; cannot rebuild a Qubicle .qbt file",
            ));
        }
    };

    let node_count = state.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbt ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let roots = state.root_hierarchy_nodes();
    let [root] = roots else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbt file needs exactly one root, but the state has {}",
            roots.len()
        )));
    };

    let root = rebuild_node(*root, state, &ext.nodes)?;

    Ok(QbtFile {
        version: ext.version,
        global_scale: ext.global_scale,
        color_map: ext
            .color_map
            .iter()
            .map(|color| QbtColor::new(color[0], color[1], color[2], color[3]))
            .collect(),
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `id` and its
/// aligned ext provenance.
fn rebuild_node(
    id: U32Id<BVoxHierarchyNode>,
    state: &VoxMain,
    nodes: &[QubicleQbtNode],
) -> Result<QbtNode> {
    let hierarchy = state
        .hierarchy_node(id)
        .ok_or_else(|| Error::invalid(format!("hierarchy node {} does not exist", id.to_u32())))?;
    let provenance = nodes.get(id.to_u32() as usize).ok_or_else(|| {
        Error::invalid(format!(
            "qubicle-qbt ext has no entry for hierarchy node {}",
            id.to_u32()
        ))
    })?;

    let node = match provenance {
        QubicleQbtNode::Model => QbtNode::Model(QbtModel {
            children: rebuild_children(hierarchy, state, nodes)?,
        }),
        QubicleQbtNode::Matrix {
            name,
            position,
            local_scale,
            pivot,
            masks,
        } => QbtNode::Matrix(matrix_from_object(
            state,
            matrix_object(hierarchy, state)?,
            name.clone(),
            *position,
            *local_scale,
            *pivot,
            masks,
        )?),
        QubicleQbtNode::Compound {
            name,
            position,
            local_scale,
            pivot,
            masks,
        } => {
            let matrix = matrix_from_object(
                state,
                matrix_object(hierarchy, state)?,
                name.clone(),
                *position,
                *local_scale,
                *pivot,
                masks,
            )?;
            QbtNode::Compound(QbtCompound {
                matrix,
                children: rebuild_children(hierarchy, state, nodes)?,
            })
        }
        QubicleQbtNode::Unknown { type_id, data } => QbtNode::Unknown(QbtUnknownNode {
            type_id: *type_id,
            data: data.clone(),
        }),
    };

    Ok(node)
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children(
    hierarchy: &VoxHierarchyNode,
    state: &VoxMain,
    nodes: &[QubicleQbtNode],
) -> Result<Vec<QbtNode>> {
    hierarchy
        .child_nodes
        .iter()
        .map(|&child| rebuild_node(child, state, nodes))
        .collect()
}

/// The build-volume object a matrix or compound node places, or an error if it
/// has none. The object is the author's build volume, so the written matrix
/// keeps the original dimensions and voxel positions directly.
fn matrix_object<'a>(hierarchy: &VoxHierarchyNode, state: &'a VoxMain) -> Result<&'a VoxObject> {
    let object_id = *hierarchy
        .child_objects
        .first()
        .ok_or_else(|| Error::invalid("a matrix or compound node has no object"))?;
    state
        .object(object_id)
        .ok_or_else(|| Error::invalid(format!("object {} does not exist", object_id.to_u32())))
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from
/// the object's `baseColorFactor` layer and its mask from the aligned ext mask
/// list, placed in `.qbt` storage order. Errors if the mask count does not
/// match the object's solid voxels.
#[allow(clippy::too_many_arguments)]
fn matrix_from_object(
    state: &VoxMain,
    object: &VoxObject,
    name: String,
    position: [i32; 3],
    local_scale: [u32; 3],
    pivot: [f32; 3],
    masks: &[u8],
) -> Result<QbtMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = bounds.to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbtVoxel::default(); volume];

    let color = object_color_ref(state, object);
    let live: Vec<_> = object.iter_live().collect();
    if live.len() != masks.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbt ext has {} masks but the object has {} solid voxels",
            masks.len(),
            live.len()
        )));
    }

    for (&voxel, &mask) in live.iter().zip(masks) {
        let position = object
            .voxel_position(voxel)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = color
            .map(|(layer, palette, binding)| {
                cell_color(state, object, voxel, layer, palette, binding)
            })
            .unwrap_or([0, 0, 0, 0]);
        // Storage order: index = y + size_y * (z + size_z * x).
        let index = position.y as usize
            + size_y as usize * (position.z as usize + size_z as usize * position.x as usize);
        voxels[index] = QbtVoxel::new(r, g, b, mask);
    }

    Ok(QbtMatrix {
        name,
        position,
        local_scale,
        pivot,
        size: [size_x, size_y, size_z],
        voxels,
    })
}
