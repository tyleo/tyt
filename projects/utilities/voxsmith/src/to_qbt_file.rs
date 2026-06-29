use crate::{Error, QubicleQbtExtWrapper, QubicleQbtNode, Result, from_vox_value, untighten};
use branded_id::U32Id;
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};
use voxcore::{
    BVoxAttribute, BVoxHierarchyNode, BVoxPaletteRef, BVoxVoxel, VoxHierarchyNode, VoxObject,
    VoxPalette, VoxState, VoxValue,
};

/// Writes a [`VoxState`] back to a decoded Qubicle Binary Tree [`QbtFile`], the
/// inverse of [`from_qbt_file`](crate::from_qbt_file).
///
/// Requires the `qubicle-qbt` ext the forward path writes; without it the file
/// cannot be rebuilt. The scene tree is walked from the single root, each matrix
/// or compound object emitting its grid with the visibility masks and color from
/// the ext and the palette.
///
/// Errors if the ext is missing, its node entries do not line up with the
/// hierarchy, the state does not have exactly one root, or a mask list does not
/// match its object.
pub fn to_qbt_file(state: &VoxState) -> Result<QbtFile> {
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

    let palette = state.iter_palettes().next().map(|(_, palette)| palette);
    let root = rebuild_node(*root, state, &ext.nodes, palette)?;

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
    state: &VoxState,
    nodes: &[QubicleQbtNode],
    palette: Option<&VoxPalette>,
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
            children: rebuild_children(hierarchy, state, nodes, palette)?,
        }),
        QubicleQbtNode::Matrix {
            name,
            position,
            local_scale,
            pivot,
            masks,
        } => QbtNode::Matrix(matrix_from_object(
            &matrix_object(hierarchy, state)?,
            palette,
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
                &matrix_object(hierarchy, state)?,
                palette,
                name.clone(),
                *position,
                *local_scale,
                *pivot,
                masks,
            )?;
            QbtNode::Compound(QbtCompound {
                matrix,
                children: rebuild_children(hierarchy, state, nodes, palette)?,
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
    state: &VoxState,
    nodes: &[QubicleQbtNode],
    palette: Option<&VoxPalette>,
) -> Result<Vec<QbtNode>> {
    hierarchy
        .child_nodes
        .iter()
        .map(|&child| rebuild_node(child, state, nodes, palette))
        .collect()
}

/// The source-grid object a matrix or compound node places, or an error if it has
/// none. The runtime object is tight; its edit grid restores the author's grid so
/// the written matrix keeps the original dimensions and voxel positions.
fn matrix_object(hierarchy: &VoxHierarchyNode, state: &VoxState) -> Result<VoxObject> {
    let object_id = *hierarchy
        .child_objects
        .first()
        .ok_or_else(|| Error::invalid("a matrix or compound node has no object"))?;
    let object = state
        .object(object_id)
        .ok_or_else(|| Error::invalid(format!("object {} does not exist", object_id.to_u32())))?;
    Ok(untighten(
        object,
        state
            .edit_object(object_id)
            .expect("a retained object has an edit grid"),
    ))
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from the
/// palette and its mask from the aligned ext mask list, placed in `.qbt` storage
/// order. Errors if the mask count does not match the object's solid voxels.
#[allow(clippy::too_many_arguments)]
fn matrix_from_object(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    name: String,
    position: [i32; 3],
    local_scale: [u32; 3],
    pivot: [f32; 3],
    masks: &[u8],
) -> Result<QbtMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = [bounds.x, bounds.y, bounds.z];
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbtVoxel::default(); volume];

    let reference = object.iter_palette_refs().next().map(|(id, _)| id);
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
        let [r, g, b] = voxel_color(object, palette, reference, voxel);
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

/// The `[r, g, b]` color a live voxel samples from the shared palette, or black
/// if the reference, cell, or `rgb` attribute is missing.
fn voxel_color(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    reference: Option<U32Id<BVoxPaletteRef>>,
    voxel: U32Id<BVoxVoxel>,
) -> [u8; 3] {
    let lookup = || -> Option<[u8; 3]> {
        let palette = palette?;
        let reference = reference?;
        let cell = object.voxel_cell(voxel, reference)?;
        let rgb = attribute_id(palette, "rgb")?;
        Some(parse_rgb(palette.cell_value(cell, rgb)))
    };
    lookup().unwrap_or([0, 0, 0])
}

/// The id of the attribute named `name`, or `None`.
fn attribute_id(palette: &VoxPalette, name: &str) -> Option<U32Id<BVoxAttribute>> {
    palette
        .iter_attributes()
        .find(|(_, attribute)| *attribute == name)
        .map(|(id, _)| id)
}

/// Parses a `#RRGGBB` color cell into `[r, g, b]`, defaulting to black on a
/// missing or malformed value.
fn parse_rgb(value: Option<&VoxValue>) -> [u8; 3] {
    let Some(VoxValue::Text(hex)) = value else {
        return [0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
            .unwrap_or(0)
    };
    [byte(0), byte(1), byte(2)]
}
