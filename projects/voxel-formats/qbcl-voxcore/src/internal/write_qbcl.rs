use crate::{
    Error, Result, SOLID_MASK,
    ext::{QbclExt, QbclExtNode, QbclExtNodeBody},
};
use branded_id::U32Id;
use qbcl::qbcl::{
    QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};
use std::collections::{HashMap, HashSet};
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Each hierarchy node's entry, paired through the listing.
type Entries<'a> = HashMap<U32Id<BVoxHierarchyNode>, &'a Option<QbclExtNode>>;

/// Writes a state to a decoded Qubicle Construction Library [`QbclFile`].
/// With `qbcl_ext`, a loaded file rebuilds exactly: the scene tree is walked
/// from the single root, each matrix or compound object emitting its grid
/// with the visibility masks from the ext and the color from the palette. A
/// node retained after the load has no entry. It emits a node like a
/// synthesized one. Without an ext, `synthesize_qbcl` builds the file from
/// the bare scene.
///
/// Errors if:
///
/// 1. the ext's node entries do not line up with the hierarchy
/// 2. the state does not have exactly one root
/// 3. a mask list does not match its object
/// 4. a model entry's node places an object under a transform chunk other
///    than the default
/// 5. an object's `baseColor` draws from a non-color value pool
pub fn write_qbcl<T>(state: &VoxMain<T>, qbcl_ext: Option<&QbclExt>) -> Result<QbclFile> {
    let Some(ext) = qbcl_ext else {
        return synthesize_qbcl(state);
    };

    let node_count = state.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qbcl ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let root_ids = state.root_hierarchy_node_ids();
    let [root_id] = root_ids else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbcl file needs exactly one root, but the state has {}",
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

    Ok(QbclFile {
        program_version: ext.program_version,
        file_version: ext.file_version,
        thumbnail: QbclThumbnail {
            width: ext.thumbnail.width,
            height: ext.thumbnail.height,
            pixels: ext
                .thumbnail
                .pixels
                .iter()
                .map(|pixel| QbclColor::new(pixel[0], pixel[1], pixel[2], pixel[3]))
                .collect(),
        },
        metadata: QbclMetadata {
            title: ext.metadata.title.clone(),
            description: ext.metadata.description.clone(),
            tags: ext.metadata.tags.clone(),
            author: ext.metadata.author.clone(),
            company: ext.metadata.company.clone(),
            website: ext.metadata.website.clone(),
            copyright: ext.metadata.copyright.clone(),
        },
        guid: ext.guid,
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its entry. The node takes the shape a synthesized node would from its
/// objects and children. The entry supplies the provenance that fits the
/// shape. A node with no entry takes a synthesized node's provenance at its
/// translation rounded to whole voxels.
fn rebuild_node<T>(
    node_id: U32Id<BVoxHierarchyNode>,
    state: &VoxMain<T>,
    entries: &Entries,
) -> Result<QbclNode> {
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

    let Some(entry) = entry else {
        return synthesized_node(state, hierarchy.name.clone(), position, &objects, children);
    };

    let body = match &entry.body {
        QbclExtNodeBody::Model { transform } => {
            if objects.is_empty() {
                QbclNodeBody::Model(QbclModel {
                    transform: model_transform(transform)?,
                    children,
                })
            } else {
                // The node now carries a grid. A matrix or compound has no
                // place for the transform chunk. Dropping the default chunk
                // loses nothing.
                if *transform != QbclModel::DEFAULT_TRANSFORM {
                    return Err(Error::invalid(
                        "a model entry's node places an object under a transform chunk other than the default",
                    ));
                }
                synthesized_body(state, position, &objects, children)?
            }
        }
        QbclExtNodeBody::Matrix {
            position,
            pivot,
            masks,
        } => entry_body(
            state,
            placed_matrix(*position, *pivot),
            masks,
            &objects,
            children,
            false,
        )?,
        QbclExtNodeBody::Compound {
            position,
            pivot,
            masks,
        } => entry_body(
            state,
            placed_matrix(*position, *pivot),
            masks,
            &objects,
            children,
            true,
        )?,
    };

    Ok(QbclNode {
        name: entry.name.clone(),
        visible: entry.visible,
        locked: entry.locked,
        body,
    })
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children<T>(
    hierarchy: &VoxHierarchyNode,
    state: &VoxMain<T>,
    entries: &Entries,
) -> Result<Vec<QbclNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, state, entries))
        .collect()
}

/// A matrix carrying an entry's placement and an empty grid.
fn placed_matrix(position: [i32; 3], pivot: [f32; 3]) -> QbclMatrix {
    QbclMatrix {
        position,
        pivot,
        ..QbclMatrix::default()
    }
}

/// The body a matrix or compound entry writes. The node's first object fills
/// `placement` under the entry's masks. A node placing no object keeps the
/// placement around an empty grid.
fn entry_body<T>(
    state: &VoxMain<T>,
    placement: QbclMatrix,
    masks: &[u8],
    objects: &[&VoxObject],
    children: Vec<QbclNode>,
    compound: bool,
) -> Result<QbclNodeBody> {
    let [first, extras @ ..] = objects else {
        // A selection dropped the grid and kept the node for its children.
        return grid_body(state, placement, &[], children, compound);
    };
    let grid = fill_grid(state, first, placement, Some(masks))?;

    grid_body(state, grid, extras, children, compound)
}

/// The body a node writes around `grid`. Qubicle has no node placing several
/// grids, so further objects become child matrices named for the object.
/// `compound` keeps a compound entry's shape when nothing else calls for one.
fn grid_body<T>(
    state: &VoxMain<T>,
    grid: QbclMatrix,
    extras: &[&VoxObject],
    children: Vec<QbclNode>,
    compound: bool,
) -> Result<QbclNodeBody> {
    let mut all_children: Vec<QbclNode> = extras
        .iter()
        .map(|object| synthesized_matrix_node(state, object, grid.position))
        .collect::<Result<_>>()?;
    all_children.extend(children);

    let body = if all_children.is_empty() && !compound {
        QbclNodeBody::Matrix(grid)
    } else {
        QbclNodeBody::Compound(QbclCompound {
            matrix: grid,
            children: all_children,
        })
    };

    Ok(body)
}

/// Fills `matrix` with `object`'s grid in `.qbcl` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer. `masks` supplies
/// each live voxel's mask in raster order. Without it every solid voxel takes
/// the solid mask. Errors if the mask count does not match the object's solid
/// voxels.
fn fill_grid<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    mut matrix: QbclMatrix,
    masks: Option<&[u8]>,
) -> Result<QbclMatrix> {
    let live_count = object.live_count();
    if let Some(masks) = masks
        && masks.len() != live_count
    {
        return Err(Error::invalid(format!(
            "qbcl ext has {} masks but the object has {live_count} solid voxels",
            masks.len()
        )));
    }

    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbclVoxel::default(); volume];

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
        voxels[index] = QbclVoxel::new(r, g, b, mask);
    }

    matrix.size = [size_x, size_y, size_z];
    matrix.voxels = voxels;

    Ok(matrix)
}

/// Converts a stored model-transform chunk into its fixed 36-byte array.
fn model_transform(bytes: &[u8]) -> Result<[u8; 36]> {
    <[u8; 36]>::try_from(bytes).map_err(|_| {
        Error::invalid(format!(
            "qbcl model transform is {} bytes, expected 36",
            bytes.len()
        ))
    })
}

/// Synthesizes a Qubicle file from the bare scene of a state written without
/// a `qbcl` ext, such as one cross-loaded from another format.
///
/// `synthesized_node` mirrors each voxcore node into Qubicle's scene tree.
/// Every voxcore root hangs under one synthetic model because Qubicle
/// requires a single root. An object placed by no node is swept under that
/// model at the origin so no geometry is dropped. An object placed by several
/// nodes is duplicated at each placement.
///
/// Lossy only where Qubicle cannot represent the source. A model's transform
/// chunk cannot carry translation, so a group node's placement is folded into
/// the world position of its descendant matrices, summed down the hierarchy
/// and rounded to whole voxels. Node rotation and scale are dropped. Colors
/// stay per voxel with no palette merge. A Qubicle voxel stores no alpha, so
/// a color's alpha is dropped. Each matrix is pivoted at its grid origin. Its
/// position is then the world coordinate of the object's min corner.
fn synthesize_qbcl<T>(state: &VoxMain<T>) -> Result<QbclFile> {
    let mut placed = HashSet::new();
    let mut children: Vec<QbclNode> = state
        .root_hierarchy_node_ids()
        .iter()
        .map(|&root_id| emit_node(state, root_id, TyVector3I32::new(0, 0, 0), &mut placed))
        .collect::<Result<_>>()?;
    for (object_id, object) in state.iter_objects() {
        if !placed.contains(&object_id) {
            children.push(synthesized_matrix_node(state, object, [0, 0, 0])?);
        }
    }

    Ok(QbclFile {
        root: QbclNode {
            name: "root".to_owned(),
            body: QbclNodeBody::Model(QbclModel {
                transform: QbclModel::DEFAULT_TRANSFORM,
                children,
            }),
            ..QbclNode::default()
        },
        ..QbclFile::default()
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
) -> Result<QbclNode> {
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
    children: Vec<QbclNode>,
) -> Result<QbclNode> {
    Ok(QbclNode {
        name,
        body: synthesized_body(state, position, objects, children)?,
        ..QbclNode::default()
    })
}

/// The body a synthesized node takes from its objects and children.
fn synthesized_body<T>(
    state: &VoxMain<T>,
    position: [i32; 3],
    objects: &[&VoxObject],
    children: Vec<QbclNode>,
) -> Result<QbclNodeBody> {
    let [first, extras @ ..] = objects else {
        return Ok(QbclNodeBody::Model(QbclModel {
            transform: QbclModel::DEFAULT_TRANSFORM,
            children,
        }));
    };
    let grid = synthesized_matrix(state, first, position)?;

    grid_body(state, grid, extras, children, false)
}

/// A matrix node named for `object` around its synthesized grid.
fn synthesized_matrix_node<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbclNode> {
    Ok(QbclNode {
        name: object.name().to_owned(),
        body: QbclNodeBody::Matrix(synthesized_matrix(state, object, position)?),
        ..QbclNode::default()
    })
}

/// `object`'s grid as a matrix with no provenance.
fn synthesized_matrix<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbclMatrix> {
    let matrix = QbclMatrix {
        position,
        ..QbclMatrix::default()
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
