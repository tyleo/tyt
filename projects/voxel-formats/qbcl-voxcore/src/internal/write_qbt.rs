use crate::{
    Error, Result, SOLID_MASK,
    ext::{QbtExt, QbtExtNode},
};
use branded_id::U32Id;
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};
use std::collections::HashSet;
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Writes a state to a decoded Qubicle Binary Tree [`QbtFile`]. With
/// `qbt_ext`, a loaded file rebuilds exactly: the scene tree is walked from
/// the single root, each matrix or compound object emitting its grid with the
/// visibility masks from the ext and the color from the palette. Without it,
/// `synthesize_qbt` builds the file from the bare scene.
///
/// Errors when the ext's node entries do not line up with the hierarchy, the
/// state does not have exactly one root, or a mask list does not match its
/// object, and when an object's `baseColor` draws from a non-color value
/// pool.
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

    let root = rebuild_node(*root_id, state, &ext.nodes)?;

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
/// and its aligned ext provenance.
fn rebuild_node<T>(
    node_id: U32Id<BVoxHierarchyNode>,
    state: &VoxMain<T>,
    nodes: &[QbtExtNode],
) -> Result<QbtNode> {
    let hierarchy = state.hierarchy_node(node_id).ok_or_else(|| {
        Error::invalid(format!(
            "hierarchy node {} does not exist",
            node_id.to_u32()
        ))
    })?;
    let provenance = nodes.get(node_id.to_u32() as usize).ok_or_else(|| {
        Error::invalid(format!(
            "qbt ext has no entry for hierarchy node {}",
            node_id.to_u32()
        ))
    })?;

    let node = match provenance {
        QbtExtNode::Model => QbtNode::Model(QbtModel {
            children: rebuild_children(hierarchy, state, nodes)?,
        }),
        QbtExtNode::Matrix {
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
        QbtExtNode::Compound {
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
        QbtExtNode::Unknown { type_id, data } => QbtNode::Unknown(QbtUnknownNode {
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
    nodes: &[QbtExtNode],
) -> Result<Vec<QbtNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, state, nodes))
        .collect()
}

/// The build-volume object a matrix or compound node places, or an error if it
/// has none. The object is the author's build volume, so the written matrix
/// keeps the original dimensions and voxel positions directly.
fn matrix_object<'a, T>(
    hierarchy: &VoxHierarchyNode,
    state: &'a VoxMain<T>,
) -> Result<&'a VoxObject> {
    let object_id = *hierarchy
        .child_object_ids
        .first()
        .ok_or_else(|| Error::invalid("a matrix or compound node has no object"))?;
    state
        .object(object_id)
        .ok_or_else(|| Error::invalid(format!("object {} does not exist", object_id.to_u32())))
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from
/// the object's `baseColor` layer and its mask from the aligned ext mask
/// list, placed in `.qbt` storage order. Errors if the mask count does not
/// match the object's solid voxels.
#[allow(clippy::too_many_arguments)]
fn matrix_from_object<T>(
    state: &VoxMain<T>,
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

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    let live_count = object.live_count();
    if live_count != masks.len() {
        return Err(Error::invalid(format!(
            "qbt ext has {} masks but the object has {live_count} solid voxels",
            masks.len()
        )));
    }

    for (voxel_id, &mask) in object.iter_live().zip(masks) {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
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

/// Synthesizes a Qubicle Binary Tree file from the bare scene of a state
/// written without a `qbt` ext, such as one cross-loaded from another format.
///
/// The voxcore hierarchy is mirrored into Qubicle's scene tree: a node with
/// only child nodes becomes a model, a node placing one object becomes a
/// matrix, and a node placing an object alongside child nodes or several
/// objects becomes a compound whose grid is the node's first object and whose
/// children hold the rest. Every voxcore root hangs under one synthetic model
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
    let mut builder = QbtBuilder::default();
    let mut children: Vec<QbtNode> = state
        .root_hierarchy_node_ids()
        .iter()
        .map(|&root_id| builder.emit_node(state, root_id, TyVector3I32::new(0, 0, 0)))
        .collect::<Result<_>>()?;
    for (object_id, object) in state.iter_objects() {
        if !builder.placed.contains(&object_id.to_u32()) {
            children.push(builder.emit_matrix_node(state, object_id, object, [0, 0, 0])?);
        }
    }

    Ok(QbtFile {
        root: QbtNode::Model(QbtModel { children }),
        ..QbtFile::default()
    })
}

/// Tracks the objects a hierarchy node has already placed while synthesizing a
/// Qubicle scene tree, so an object no node places can be swept in once.
#[derive(Default)]
struct QbtBuilder {
    placed: HashSet<u32>,
}

impl QbtBuilder {
    /// Maps one hierarchy node and its subtree to a Qubicle node. The node's
    /// translation sums into the world position because a model node cannot
    /// carry it. The node's first object rides on the node itself as a matrix
    /// or compound grid. Any further objects and the mapped child nodes become
    /// its children.
    fn emit_node<T>(
        &mut self,
        state: &VoxMain<T>,
        node_id: U32Id<BVoxHierarchyNode>,
        parent: TyVector3I32,
    ) -> Result<QbtNode> {
        let (name, child_object_ids, child_node_ids, world) = {
            let node = state
                .hierarchy_node(node_id)
                .expect("a hierarchy id from the state resolves");
            let position = node.transform.position;
            let world = parent + position.round().as_ivec3();
            (
                node.name.clone(),
                node.child_object_ids.clone(),
                node.child_node_ids.clone(),
                world,
            )
        };

        let objects: Vec<(U32Id<BVoxObject>, &VoxObject)> = child_object_ids
            .iter()
            .filter_map(|&object_id| state.object(object_id).map(|object| (object_id, object)))
            .collect();
        let mut objects = objects.into_iter();
        let first = objects.next();

        let mut children: Vec<QbtNode> = objects
            .map(|(object_id, object)| {
                self.emit_matrix_node(state, object_id, object, world.to_array())
            })
            .collect::<Result<_>>()?;
        for child_id in child_node_ids {
            children.push(self.emit_node(state, child_id, world)?);
        }

        let node = match first {
            None => QbtNode::Model(QbtModel { children }),
            // The object is the author's build volume, so the matrix keeps its
            // dimensions and voxel positions directly.
            Some((object_id, object)) if children.is_empty() => QbtNode::Matrix(
                self.synthesize_matrix(object_id, object, name, world.to_array(), state)?,
            ),
            Some((object_id, object)) => QbtNode::Compound(QbtCompound {
                matrix: self.synthesize_matrix(object_id, object, name, world.to_array(), state)?,
                children,
            }),
        };

        Ok(node)
    }

    /// Wraps one object in a matrix node placed at `world`, naming the matrix
    /// for the object. Used for a node's extra objects and for the
    /// unplaced-object sweep.
    fn emit_matrix_node<T>(
        &mut self,
        state: &VoxMain<T>,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        world: [i32; 3],
    ) -> Result<QbtNode> {
        let name = object.name().to_owned();

        // The object is the author's build volume, so the matrix keeps its
        // dimensions and voxel positions directly.
        Ok(QbtNode::Matrix(self.synthesize_matrix(
            object_id, object, name, world, state,
        )?))
    }

    /// Builds the matrix grid for `object` at the world `position`, pivoted
    /// at its grid origin at unit local scale, and marks the object placed so
    /// the sweep skips it.
    fn synthesize_matrix<T>(
        &mut self,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        name: String,
        position: [i32; 3],
        state: &VoxMain<T>,
    ) -> Result<QbtMatrix> {
        self.placed.insert(object_id.to_u32());

        let bounds = object.bounds();
        let [size_x, size_y, size_z] = bounds.to_array();
        let volume = size_x as usize * size_y as usize * size_z as usize;
        let mut voxels = vec![QbtVoxel::default(); volume];

        let cell_color = resolve_cell_color_or_transparent(state, object)?;
        for voxel_id in object.iter_live() {
            let cell = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            // A Qubicle voxel stores no alpha, so the sampled color's alpha is
            // dropped.
            let [r, g, b, _] = cell_color.color(voxel_id);
            // Storage order: index = y + size_y * (z + size_z * x).
            let index = cell.y as usize
                + size_y as usize * (cell.z as usize + size_z as usize * cell.x as usize);
            voxels[index] = QbtVoxel::new(r, g, b, SOLID_MASK);
        }

        Ok(QbtMatrix {
            name,
            position,
            local_scale: [1, 1, 1],
            pivot: [0.0, 0.0, 0.0],
            size: [size_x, size_y, size_z],
            voxels,
        })
    }
}
