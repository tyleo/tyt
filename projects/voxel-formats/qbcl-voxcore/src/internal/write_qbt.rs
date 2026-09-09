use crate::{
    Error, Result, SOLID_MASK,
    ext::{QbtExt, QbtExtNode},
};
use branded_id::U32Id;
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};
use std::collections::{HashMap, HashSet};
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Each hierarchy node's entry, paired through the listing.
type Entries<'a> = HashMap<U32Id<BVoxHierarchyNode>, &'a Option<QbtExtNode>>;

/// Writes a state to a decoded Qubicle Binary Tree [`QbtFile`]. With
/// `qbt_ext`, a loaded file rebuilds exactly: the scene tree is walked from
/// the single root, each matrix or compound object emitting its grid with the
/// visibility masks from the ext and the color from the palette. A node
/// retained after the load has no entry. It emits a node like a synthesized
/// one. Without an ext, `synthesize_qbt` builds the file from the bare scene.
///
/// Errors if:
///
/// 1. the ext's node entries do not line up with the hierarchy
/// 2. the state does not have exactly one root
/// 3. a mask list does not match its object
/// 4. an object's `baseColor` draws from a non-color value pool
pub fn write_qbt<T>(state: &VoxMain<T>, qbt_ext: Option<&QbtExt>) -> Result<QbtFile> {
    let Some(ext) = qbt_ext else {
        return synthesize_qbt(state);
    };

    let node_count = state.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qbt ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let root_ids = state.root_hierarchy_node_ids();
    let [root_id] = root_ids else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbt file needs exactly one root, but the state has {}",
            root_ids.len()
        )));
    };

    // The entries follow the listing. An id matches its index only once
    // `gc` has renumbered.
    let entries: Entries = state
        .iter_hierarchy_nodes()
        .zip(&ext.nodes)
        .map(|((node_id, _), entry)| (node_id, entry))
        .collect();

    let root = rebuild_node(*root_id, state, &entries)?;

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

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its entry. A node with no entry takes the shape of a synthesized node
/// at its translation rounded to whole voxels. A model entry holds nothing,
/// so its node takes the same shape. A matrix or compound entry whose node
/// places no object keeps its placement around an empty grid.
fn rebuild_node<T>(
    node_id: U32Id<BVoxHierarchyNode>,
    state: &VoxMain<T>,
    entries: &Entries,
) -> Result<QbtNode> {
    let hierarchy = state
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let entry = entries
        .get(&node_id)
        .expect("the count check paired every node with an entry");

    let node = match entry {
        None | Some(QbtExtNode::Model) => {
            // The reader made each position its node's translation, so the
            // translation goes back as the position. The synthesizer's
            // summing has no place here because the ancestors carry their
            // positions in their entries.
            let position = rounded_translation(hierarchy).to_array();
            let children = rebuild_children(hierarchy, state, entries)?;
            synthesized_node(
                state,
                hierarchy.name.clone(),
                position,
                &placed_objects(hierarchy, state),
                children,
            )?
        }
        Some(QbtExtNode::Matrix {
            name,
            position,
            local_scale,
            pivot,
            masks,
        }) => QbtNode::Matrix(entry_matrix(
            state,
            hierarchy,
            placed_matrix(name, *position, *local_scale, *pivot),
            masks,
        )?),
        Some(QbtExtNode::Compound {
            name,
            position,
            local_scale,
            pivot,
            masks,
        }) => QbtNode::Compound(QbtCompound {
            matrix: entry_matrix(
                state,
                hierarchy,
                placed_matrix(name, *position, *local_scale, *pivot),
                masks,
            )?,
            children: rebuild_children(hierarchy, state, entries)?,
        }),
        Some(QbtExtNode::Unknown { type_id, data }) => QbtNode::Unknown(QbtUnknownNode {
            type_id: *type_id,
            data: data.clone(),
        }),
    };

    Ok(node)
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children<T>(
    hierarchy: &VoxHierarchyNode,
    state: &VoxMain<T>,
    entries: &Entries,
) -> Result<Vec<QbtNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, state, entries))
        .collect()
}

/// A matrix carrying an entry's provenance and an empty grid.
fn placed_matrix(
    name: &str,
    position: [i32; 3],
    local_scale: [u32; 3],
    pivot: [f32; 3],
) -> QbtMatrix {
    QbtMatrix {
        name: name.to_owned(),
        position,
        local_scale,
        pivot,
        ..QbtMatrix::default()
    }
}

/// The grid a matrix or compound entry writes: the node's first object
/// filled into `matrix` under the entry's masks. The object is the author's
/// build volume, so the written matrix keeps its dimensions and voxel
/// positions directly.
fn entry_matrix<T>(
    state: &VoxMain<T>,
    hierarchy: &VoxHierarchyNode,
    matrix: QbtMatrix,
    masks: &[u8],
) -> Result<QbtMatrix> {
    let Some(&object_id) = hierarchy.child_object_ids.first() else {
        // A selection dropped the grid and kept the node for its children.
        // The placement survives around an empty grid.
        return Ok(matrix);
    };
    let object = state
        .object(object_id)
        .expect("a placed object is one of the state's");

    fill_grid(state, object, matrix, Some(masks))
}

/// Fills `matrix` with `object`'s grid in `.qbt` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer. `masks` supplies
/// each live voxel's mask in raster order. Without it every solid voxel takes
/// the solid mask. Errors if the mask count does not match the object's solid
/// voxels.
fn fill_grid<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    mut matrix: QbtMatrix,
    masks: Option<&[u8]>,
) -> Result<QbtMatrix> {
    let live_count = object.live_count();
    if let Some(masks) = masks
        && masks.len() != live_count
    {
        return Err(Error::invalid(format!(
            "qbt ext has {} masks but the object has {live_count} solid voxels",
            masks.len()
        )));
    }

    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbtVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    for (live_index, voxel_id) in object.iter_live().enumerate() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        let mask = masks.map_or(SOLID_MASK, |masks| masks[live_index]);
        // Storage order: index = y + size_y * (z + size_z * x).
        let index = position.y as usize
            + size_y as usize * (position.z as usize + size_z as usize * position.x as usize);
        voxels[index] = QbtVoxel::new(r, g, b, mask);
    }

    matrix.size = [size_x, size_y, size_z];
    matrix.voxels = voxels;

    Ok(matrix)
}

/// Synthesizes a Qubicle Binary Tree file from the bare scene of a state
/// written without a `qbt` ext, such as one cross-loaded from another format.
///
/// `synthesized_node` mirrors each voxcore node into Qubicle's scene tree.
/// Every voxcore root hangs under one synthetic model
/// because Qubicle requires a single root. An object placed by no node is
/// swept under that model at the origin so no geometry is dropped. An object
/// placed by several nodes is duplicated at each placement.
///
/// Lossy only where Qubicle cannot represent the source. A model node carries
/// no name or translation: a group node's name is dropped, and its placement
/// is folded into the world position of its descendant matrices, summed down
/// the hierarchy and rounded to whole voxels. Node rotation and scale are
/// dropped. Colors stay per voxel with no palette merge. A Qubicle voxel
/// stores no alpha, so a color's alpha is dropped. Each matrix is pivoted at
/// its grid origin at unit local scale. Its position is then the world
/// coordinate of the object's min corner.
fn synthesize_qbt<T>(state: &VoxMain<T>) -> Result<QbtFile> {
    let mut placed = HashSet::new();
    let mut children: Vec<QbtNode> = state
        .root_hierarchy_node_ids()
        .iter()
        .map(|&root_id| emit_node(state, root_id, TyVector3I32::new(0, 0, 0), &mut placed))
        .collect::<Result<_>>()?;
    for (object_id, object) in state.iter_objects() {
        if !placed.contains(&object_id) {
            let matrix = synthesized_matrix(state, object, object.name().to_owned(), [0, 0, 0])?;
            children.push(QbtNode::Matrix(matrix));
        }
    }

    Ok(QbtFile {
        root: QbtNode::Model(QbtModel { children }),
        ..QbtFile::default()
    })
}

/// Maps one hierarchy node and its subtree to a Qubicle node. The
/// translations sum into the world position because a model node cannot
/// carry one. The node's objects go into `placed`.
fn emit_node<T>(
    state: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: TyVector3I32,
    placed: &mut HashSet<U32Id<BVoxObject>>,
) -> Result<QbtNode> {
    let node = state
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let world = parent + rounded_translation(node);
    placed.extend(node.child_object_ids.iter().copied());

    let children = node
        .child_node_ids
        .iter()
        .map(|&child_id| emit_node(state, child_id, world, placed))
        .collect::<Result<_>>()?;

    synthesized_node(
        state,
        node.name.clone(),
        world.to_array(),
        &placed_objects(node, state),
        children,
    )
}

/// The Qubicle node a hierarchy node synthesizes to. Qubicle has no node
/// placing several grids, so further objects become child matrices named for
/// the object.
fn synthesized_node<T>(
    state: &VoxMain<T>,
    name: String,
    position: [i32; 3],
    objects: &[&VoxObject],
    children: Vec<QbtNode>,
) -> Result<QbtNode> {
    let [first, extras @ ..] = objects else {
        return Ok(QbtNode::Model(QbtModel { children }));
    };

    let mut all_children: Vec<QbtNode> = extras
        .iter()
        .map(|object| {
            synthesized_matrix(state, object, object.name().to_owned(), position)
                .map(QbtNode::Matrix)
        })
        .collect::<Result<_>>()?;
    all_children.extend(children);

    let matrix = synthesized_matrix(state, first, name, position)?;
    let node = if all_children.is_empty() {
        QbtNode::Matrix(matrix)
    } else {
        QbtNode::Compound(QbtCompound {
            matrix,
            children: all_children,
        })
    };

    Ok(node)
}

/// `object`'s grid as a matrix with no provenance.
fn synthesized_matrix<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    name: String,
    position: [i32; 3],
) -> Result<QbtMatrix> {
    let matrix = QbtMatrix {
        name,
        position,
        local_scale: [1, 1, 1],
        ..QbtMatrix::default()
    };

    fill_grid(state, object, matrix, None)
}

/// The objects a hierarchy node places, in order.
fn placed_objects<'a, T>(
    hierarchy: &VoxHierarchyNode,
    state: &'a VoxMain<T>,
) -> Vec<&'a VoxObject> {
    hierarchy
        .child_object_ids
        .iter()
        .map(|&object_id| {
            state
                .object(object_id)
                .expect("a placed object is one of the state's")
        })
        .collect()
}

/// A node's translation rounded to a Qubicle position's whole voxels.
fn rounded_translation(node: &VoxHierarchyNode) -> TyVector3I32 {
    node.transform.position.round().as_ivec3()
}
