use crate::{
    Error, QubicleQbclExtWrapper, QubicleQbclNode, QubicleQbclNodeBody, Result, ext_for,
    from_vox_value, resolve_cell_color_or_transparent,
};
use branded_id::U32Id;
use qbcl::qbcl::{
    QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};
use std::collections::HashSet;
use ty_math::TyVector3I32;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

/// Writes a [`VoxMain`] to a decoded Qubicle Construction Library [`QbclFile`].
///
/// When the state carries the `qubicle-qbcl` ext the forward path writes, the
/// file is rebuilt from it exactly, the inverse of
/// [`from_qbcl_file`](crate::from_qbcl_file): the scene tree is walked from the
/// single root, each matrix or compound emitting its grid with the visibility
/// masks and color from the ext and the palette. When the ext is absent or
/// names another format, the file is synthesized from the bare scene by
/// `synthesize_qbcl`, so any source can be written to Qubicle.
///
/// Errors when a `qubicle-qbcl` ext is present but its node entries do not
/// line up with the hierarchy, the state does not have exactly one root, or a
/// mask list does not match its object, and when an object's
/// `baseColorFactor` draws from a non-color pool.
pub fn to_qbcl_file(state: &VoxMain) -> Result<QbclFile> {
    let ext = match ext_for(state, "qubicle-qbcl") {
        Some(ext) => from_vox_value::<QubicleQbclExtWrapper>(ext)?.qubicle_qbcl,
        None => return synthesize_qbcl(state),
    };

    let node_count = state.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbcl ext has {} nodes but the state has {node_count} hierarchy nodes",
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

    let root = rebuild_node(*root_id, state, &ext.nodes)?;

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
            title: ext.metadata.title,
            description: ext.metadata.description,
            tags: ext.metadata.tags,
            author: ext.metadata.author,
            company: ext.metadata.company,
            website: ext.metadata.website,
            copyright: ext.metadata.copyright,
        },
        guid: ext.guid,
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its aligned ext provenance.
fn rebuild_node(
    node_id: U32Id<BVoxHierarchyNode>,
    state: &VoxMain,
    nodes: &[QubicleQbclNode],
) -> Result<QbclNode> {
    let hierarchy = state.hierarchy_node(node_id).ok_or_else(|| {
        Error::invalid(format!(
            "hierarchy node {} does not exist",
            node_id.to_u32()
        ))
    })?;
    let provenance = nodes.get(node_id.to_u32() as usize).ok_or_else(|| {
        Error::invalid(format!(
            "qubicle-qbcl ext has no entry for hierarchy node {}",
            node_id.to_u32()
        ))
    })?;

    let body = match &provenance.body {
        QubicleQbclNodeBody::Model { transform } => QbclNodeBody::Model(QbclModel {
            transform: model_transform(transform)?,
            children: rebuild_children(hierarchy, state, nodes)?,
        }),
        QubicleQbclNodeBody::Matrix {
            position,
            pivot,
            masks,
        } => QbclNodeBody::Matrix(matrix_from_object(
            state,
            matrix_object(hierarchy, state)?,
            *position,
            *pivot,
            masks,
        )?),
        QubicleQbclNodeBody::Compound {
            position,
            pivot,
            masks,
        } => {
            let matrix = matrix_from_object(
                state,
                matrix_object(hierarchy, state)?,
                *position,
                *pivot,
                masks,
            )?;
            QbclNodeBody::Compound(QbclCompound {
                matrix,
                children: rebuild_children(hierarchy, state, nodes)?,
            })
        }
    };

    Ok(QbclNode {
        name: provenance.name.clone(),
        visible: provenance.visible,
        locked: provenance.locked,
        body,
    })
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children(
    hierarchy: &VoxHierarchyNode,
    state: &VoxMain,
    nodes: &[QubicleQbclNode],
) -> Result<Vec<QbclNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, state, nodes))
        .collect()
}

/// The build-volume object a matrix or compound node places, or an error if it
/// has none. The object is the author's build volume, so the written matrix
/// keeps the original dimensions and voxel positions directly.
fn matrix_object<'a>(hierarchy: &VoxHierarchyNode, state: &'a VoxMain) -> Result<&'a VoxObject> {
    let object_id = *hierarchy
        .child_object_ids
        .first()
        .ok_or_else(|| Error::invalid("a matrix or compound node has no object"))?;
    state
        .object(object_id)
        .ok_or_else(|| Error::invalid(format!("object {} does not exist", object_id.to_u32())))
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from
/// the object's `baseColorFactor` layer and its mask from the aligned ext mask
/// list, placed in `.qbcl` storage order. Errors if the mask count does not
/// match the object's solid voxels.
fn matrix_from_object(
    state: &VoxMain,
    object: &VoxObject,
    position: [i32; 3],
    pivot: [f32; 3],
    masks: &[u8],
) -> Result<QbclMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = bounds.to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbclVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    let live_count = object.live_count();
    if live_count != masks.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbcl ext has {} masks but the object has {live_count} solid voxels",
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
        voxels[index] = QbclVoxel::new(r, g, b, mask);
    }

    Ok(QbclMatrix {
        size: [size_x, size_y, size_z],
        position,
        pivot,
        voxels,
    })
}

/// Converts a stored model-transform chunk into its fixed 36-byte array.
fn model_transform(bytes: &[u8]) -> Result<[u8; 36]> {
    <[u8; 36]>::try_from(bytes).map_err(|_| {
        Error::invalid(format!(
            "qubicle-qbcl model transform is {} bytes, expected 36",
            bytes.len()
        ))
    })
}

/// The visibility mask written for every synthesized solid voxel. Qubicle reads
/// a zero mask as an empty cell and any non-zero mask as a solid voxel whose
/// bits are a per-face visibility set; the exact bits are cosmetic, so this
/// mirrors the codec's solid fixture.
const SOLID_MASK: u8 = 0x7e;

/// Synthesizes a Qubicle file from a state that carries no `qubicle-qbcl` ext,
/// such as one cross-loaded from another format.
///
/// The voxcore hierarchy is mirrored into Qubicle's scene tree: a node with
/// only child nodes becomes a model, a node placing one object becomes a
/// matrix, and a node placing an object alongside child nodes or several
/// objects becomes a compound whose grid is the node's first object and whose
/// children hold the rest. Qubicle requires a single root, so every voxcore
/// root hangs under one synthetic model; an object no node places is swept
/// under it at the origin so no geometry is dropped, and an object placed by
/// several nodes is duplicated at each placement.
///
/// Lossy only where Qubicle cannot represent the source: a model's transform
/// chunk cannot carry translation, so a group node's placement is folded into
/// the world position of its descendant matrices, summed down the hierarchy and
/// rounded to whole voxels, and node rotation and scale are dropped. Colors
/// stay per voxel with no palette merge, but a Qubicle voxel stores no alpha,
/// so a color's alpha is dropped. Each matrix is pivoted at its grid origin, so
/// its position is the world coordinate of the object's min corner.
fn synthesize_qbcl(state: &VoxMain) -> Result<QbclFile> {
    let mut builder = QbclBuilder::default();
    let mut children: Vec<QbclNode> = state
        .root_hierarchy_node_ids()
        .iter()
        .map(|&root_id| builder.emit_node(state, root_id, TyVector3I32::new(0, 0, 0)))
        .collect::<Result<_>>()?;
    for (object_id, object) in state.iter_objects() {
        if !builder.placed.contains(&object_id.to_u32()) {
            children.push(builder.emit_object_node(state, object_id, object, [0, 0, 0])?);
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

/// Tracks the objects a hierarchy node has already placed while synthesizing a
/// Qubicle scene tree, so an object no node places can be swept in once.
#[derive(Default)]
struct QbclBuilder {
    placed: HashSet<u32>,
}

impl QbclBuilder {
    /// Maps one hierarchy node and its subtree to a Qubicle node, summing the
    /// node's translation into the world position so descendant matrices land
    /// correctly even though a model node cannot carry translation. The node's
    /// first object rides on the node itself as a matrix or compound grid; any
    /// further objects and the mapped child nodes become its children.
    fn emit_node(
        &mut self,
        state: &VoxMain,
        node_id: U32Id<BVoxHierarchyNode>,
        parent: TyVector3I32,
    ) -> Result<QbclNode> {
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

        let mut children: Vec<QbclNode> = objects
            .map(|(object_id, object)| {
                self.emit_object_node(state, object_id, object, world.to_array())
            })
            .collect::<Result<_>>()?;
        for child_id in child_node_ids {
            children.push(self.emit_node(state, child_id, world)?);
        }

        let body = match first {
            None => QbclNodeBody::Model(QbclModel {
                transform: QbclModel::DEFAULT_TRANSFORM,
                children,
            }),
            // The object is the author's build volume, so the matrix keeps its
            // dimensions and voxel positions directly.
            Some((object_id, object)) if children.is_empty() => QbclNodeBody::Matrix(
                self.synthesize_matrix(object_id, object, world.to_array(), state)?,
            ),
            Some((object_id, object)) => QbclNodeBody::Compound(QbclCompound {
                matrix: self.synthesize_matrix(object_id, object, world.to_array(), state)?,
                children,
            }),
        };

        Ok(QbclNode {
            name,
            body,
            ..QbclNode::default()
        })
    }

    /// Wraps one object in a matrix node placed at `world`, naming the node for
    /// the object. Used for a node's extra objects and for the unplaced-object
    /// sweep.
    fn emit_object_node(
        &mut self,
        state: &VoxMain,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        world: [i32; 3],
    ) -> Result<QbclNode> {
        // The object is the author's build volume, so the matrix keeps its
        // dimensions and voxel positions directly.
        Ok(QbclNode {
            name: object.name().to_owned(),
            body: QbclNodeBody::Matrix(self.synthesize_matrix(object_id, object, world, state)?),
            ..QbclNode::default()
        })
    }

    /// Builds a matrix grid from an object, placed at the world `position`: one
    /// solid voxel per live cell in `.qbcl` storage order, each carrying the
    /// object's color with its alpha dropped and a solid visibility mask. Marks
    /// the object placed so the sweep does not re-emit it.
    fn synthesize_matrix(
        &mut self,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        position: [i32; 3],
        state: &VoxMain,
    ) -> Result<QbclMatrix> {
        self.placed.insert(object_id.to_u32());

        let bounds = object.bounds();
        let [size_x, size_y, size_z] = bounds.to_array();
        let volume = size_x as usize * size_y as usize * size_z as usize;
        let mut voxels = vec![QbclVoxel::default(); volume];

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
            voxels[index] = QbclVoxel::new(r, g, b, SOLID_MASK);
        }

        Ok(QbclMatrix {
            size: [size_x, size_y, size_z],
            position,
            pivot: [0.0, 0.0, 0.0],
            voxels,
        })
    }
}
