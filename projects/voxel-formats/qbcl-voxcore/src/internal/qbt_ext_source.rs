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
    BVoxHierarchyNode, BVoxObject, VoxExt, VoxHierarchyNode, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Writes a state to a Qubicle Binary Tree file. [`QbtExt`] rebuilds the
/// loaded file. `()` synthesizes one from the scene.
pub trait QbtExtSource: VoxExt + Sized {
    /// Writes `state` to a [`QbtFile`].
    fn write_qbt(state: &VoxMain<Self>) -> Result<QbtFile>;
}

impl QbtExtSource for QbtExt {
    /// Writes a state to a decoded Qubicle Binary Tree [`QbtFile`] through its
    /// ext, so a loaded file rebuilds exactly: the scene tree is walked from
    /// the single root, each matrix or compound object emitting its grid with
    /// the visibility masks from the ext and the color from the palette. A node
    /// retained after the load has no entry. It emits a node like a synthesized
    /// one.
    ///
    /// Errors if:
    ///
    /// 1. the ext's node entries do not line up with the hierarchy
    /// 2. the state does not have exactly one root
    /// 3. a mask list does not match its object
    /// 4. an unknown entry's node places an object or lists a child node
    /// 5. an object's `baseColor` draws from a non-color value pool
    fn write_qbt(state: &VoxMain<Self>) -> Result<QbtFile> {
        let ext = state.ext();

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
}

/// Each hierarchy node's entry, paired through the listing.
type Entries<'a> = HashMap<U32Id<BVoxHierarchyNode>, &'a Option<QbtExtNode>>;

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its entry. The node takes the shape a synthesized node would from its
/// objects and children. The entry supplies the provenance that fits the
/// shape. A node with no entry takes a synthesized node's provenance at its
/// translation rounded to whole voxels. A model entry holds nothing, so its
/// node takes the same provenance. An unknown entry fits only an unknown
/// node.
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

    // The reader made each position its node's translation, so the
    // translation goes back as the position. The synthesizer's summing has
    // no place here because the ancestors carry their positions in their
    // entries.
    let position = rounded_translation(hierarchy).to_array();
    let objects = placed_objects(hierarchy, state);
    let children = rebuild_children(hierarchy, state, entries)?;

    let node = match entry {
        None | Some(QbtExtNode::Model) => {
            synthesized_node(state, hierarchy.name.clone(), position, &objects, children)?
        }
        Some(QbtExtNode::Matrix {
            name,
            position,
            local_scale,
            pivot,
            masks,
        }) => entry_node(
            state,
            placed_matrix(name, *position, *local_scale, *pivot),
            masks,
            &objects,
            children,
            false,
        )?,
        Some(QbtExtNode::Compound {
            name,
            position,
            local_scale,
            pivot,
            masks,
        }) => entry_node(
            state,
            placed_matrix(name, *position, *local_scale, *pivot),
            masks,
            &objects,
            children,
            true,
        )?,
        Some(QbtExtNode::Unknown { type_id, data }) => {
            // Opaque bytes have no place for a grid or a child.
            if !objects.is_empty() || !children.is_empty() {
                return Err(Error::invalid(
                    "an unknown entry's node places an object or lists a child node",
                ));
            }
            QbtNode::Unknown(QbtUnknownNode {
                type_id: *type_id,
                data: data.clone(),
            })
        }
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

/// The node a matrix or compound entry writes. The node's first object fills
/// `placement` under the entry's masks. A node placing no object keeps the
/// placement around an empty grid.
fn entry_node<T>(
    state: &VoxMain<T>,
    placement: QbtMatrix,
    masks: &[u8],
    objects: &[&VoxObject],
    children: Vec<QbtNode>,
    compound: bool,
) -> Result<QbtNode> {
    let [first, extras @ ..] = objects else {
        // A selection dropped the grid and kept the node for its children.
        return grid_node(state, placement, &[], children, compound);
    };
    let grid = fill_grid(state, first, placement, Some(masks))?;

    grid_node(state, grid, extras, children, compound)
}

impl QbtExtSource for () {
    /// Synthesizes a Qubicle Binary Tree file from the bare scene of a state
    /// carrying no `qbt` ext, such as one cross-loaded from another format.
    ///
    /// `synthesized_node` mirrors each voxcore node into Qubicle's scene tree.
    /// Every voxcore root hangs under one synthetic model because Qubicle
    /// requires a single root. An object placed by no node is swept under that
    /// model at the origin so no geometry is dropped. An object placed by
    /// several nodes is duplicated at each placement.
    ///
    /// Lossy only where Qubicle cannot represent the source. A model node
    /// carries no name or translation: a group node's name is dropped, and its
    /// placement is folded into the world position of its descendant matrices,
    /// summed down the hierarchy and rounded to whole voxels. Node rotation and
    /// scale are dropped. Colors stay per voxel with no palette merge. A
    /// color's alpha drops because a Qubicle voxel stores no alpha. Each matrix
    /// is pivoted at its grid origin at unit local scale. Its position is then
    /// the world coordinate of the object's min corner.
    fn write_qbt(state: &VoxMain<Self>) -> Result<QbtFile> {
        let mut placed = HashSet::new();
        let mut children: Vec<QbtNode> = state
            .root_hierarchy_node_ids()
            .iter()
            .map(|&root_id| emit_node(state, root_id, TyVector3I32::new(0, 0, 0), &mut placed))
            .collect::<Result<_>>()?;
        for (object_id, object) in state.iter_objects() {
            if !placed.contains(&object_id) {
                children.push(synthesized_matrix_node(state, object, [0, 0, 0])?);
            }
        }

        Ok(QbtFile {
            root: QbtNode::Model(QbtModel { children }),
            ..QbtFile::default()
        })
    }
}

/// The node written around `grid`. Qubicle has no node placing several grids,
/// so further objects become child matrices named for the object. `compound`
/// keeps a compound entry's shape when nothing else calls for one.
fn grid_node<T>(
    state: &VoxMain<T>,
    grid: QbtMatrix,
    extras: &[&VoxObject],
    children: Vec<QbtNode>,
    compound: bool,
) -> Result<QbtNode> {
    let mut all_children: Vec<QbtNode> = extras
        .iter()
        .map(|object| synthesized_matrix_node(state, object, grid.position))
        .collect::<Result<_>>()?;
    all_children.extend(children);

    let node = if all_children.is_empty() && !compound {
        QbtNode::Matrix(grid)
    } else {
        QbtNode::Compound(QbtCompound {
            matrix: grid,
            children: all_children,
        })
    };

    Ok(node)
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

/// The Qubicle node a hierarchy node synthesizes to.
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
    let grid = synthesized_matrix(state, first, name, position)?;

    grid_node(state, grid, extras, children, false)
}

/// A matrix node named for `object` around its synthesized grid.
fn synthesized_matrix_node<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbtNode> {
    synthesized_matrix(state, object, object.name().to_owned(), position).map(QbtNode::Matrix)
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
