use crate::{
    Error, Result,
    ext::{QbExt, QbExtMatrix},
};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use std::collections::HashSet;
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxMain, VoxObject, color::resolve_cell_color_or_transparent,
};

/// Writes a state to a decoded Qubicle Binary [`QbFile`]. With `qb_ext`, a
/// loaded file rebuilds exactly: each object emits one matrix, taking its
/// name, position, and per-voxel visibility from the ext and its colors from
/// the palette. Without it, `synthesize_qb` builds the file from the bare
/// scene.
///
/// Errors when the ext's matrix entries do not line up with the objects or a
/// visibility list does not match its object, and when an object's
/// `baseColor` draws from a non-color value pool.
pub fn write_qb<T>(state: &VoxMain<T>, qb_ext: Option<&QbExt>) -> Result<QbFile> {
    let Some(ext) = qb_ext else {
        return synthesize_qb(state);
    };

    let object_count = state.object_count();
    if object_count != ext.matrices.len() {
        return Err(Error::invalid(format!(
            "qb ext has {} matrices but the state has {object_count} objects",
            ext.matrices.len()
        )));
    }

    // The object is the author's build volume, so the written matrix keeps its
    // dimensions and voxel positions directly.
    let matrices = state
        .iter_objects()
        .zip(&ext.matrices)
        .map(|((_, object), provenance)| matrix_from_object(state, object, provenance))
        .collect::<Result<_>>()?;

    Ok(QbFile {
        version: ext.version,
        color_format: if ext.bgra {
            QbColorFormat::Bgra
        } else {
            QbColorFormat::Rgba
        },
        z_axis_orientation: if ext.right_handed {
            QbZAxisOrientation::RightHanded
        } else {
            QbZAxisOrientation::LeftHanded
        },
        compressed: ext.compressed,
        visibility_mask_encoded: ext.visibility_mask_encoded,
        matrices,
    })
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from
/// the object's `baseColor` layer and its visibility from the aligned ext
/// list, placed in `.qb` storage order. Errors if the visibility count does not
/// match the object's solid voxels.
fn matrix_from_object<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    provenance: &QbExtMatrix,
) -> Result<QbMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = bounds.to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    let live_count = object.live_count();
    if live_count != provenance.visibility.len() {
        return Err(Error::invalid(format!(
            "qb ext has {} visibility bytes but the object has {live_count} solid voxels",
            provenance.visibility.len()
        )));
    }

    for (voxel_id, &visibility) in object.iter_live().zip(&provenance.visibility) {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        // Storage order: index = x + size_x * (y + size_y * z).
        let index = position.x as usize
            + size_x as usize * (position.y as usize + size_y as usize * position.z as usize);
        voxels[index] = QbVoxel {
            r,
            g,
            b,
            visibility,
        };
    }

    Ok(QbMatrix {
        name: provenance.name.clone(),
        size: [size_x, size_y, size_z],
        position: provenance.position,
        voxels,
    })
}

/// Synthesizes a Qubicle Binary file from the bare scene of a state written
/// without a `qb` ext, such as one cross-loaded from another format.
///
/// Qubicle Binary has a flat matrix list and no hierarchy. Every object
/// placement becomes one matrix at the placement's world translation, summed
/// down the hierarchy from the roots and rounded to whole voxels. A node's
/// first object takes the node's name and any further object its own name.
/// An object placed by no node is emitted once at the origin so no geometry
/// is dropped. An object placed by several nodes is duplicated at each
/// placement.
///
/// Lossy only where Qubicle Binary cannot represent the source: grouping
/// collapses, node rotation and scale are dropped, and a color's alpha is
/// dropped because a Qubicle voxel stores none. The header is the default:
/// `RGBA`, left-handed, uncompressed, with a plain solid visibility byte.
fn synthesize_qb<T>(state: &VoxMain<T>) -> Result<QbFile> {
    let mut builder = QbBuilder::default();
    for &root_id in state.root_hierarchy_node_ids() {
        builder.emit_node(state, root_id, TyVector3I32::new(0, 0, 0))?;
    }
    for (object_id, object) in state.iter_objects() {
        if !builder.placed.contains(&object_id.to_u32()) {
            builder.emit_matrix(state, object_id, object, [0, 0, 0], object.name())?;
        }
    }

    Ok(QbFile {
        matrices: builder.matrices,
        ..QbFile::default()
    })
}

/// Accumulates the flattened Qubicle Binary scene. `placed` lets the sweep
/// emit an object no node places once.
#[derive(Default)]
struct QbBuilder {
    matrices: Vec<QbMatrix>,
    placed: HashSet<u32>,
}

impl QbBuilder {
    /// Walks one hierarchy node and its subtree, summing translations into
    /// the world position each matrix carries.
    fn emit_node<T>(
        &mut self,
        state: &VoxMain<T>,
        node_id: U32Id<BVoxHierarchyNode>,
        parent: TyVector3I32,
    ) -> Result<()> {
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

        for (index, object_id) in child_object_ids.into_iter().enumerate() {
            let Some(object) = state.object(object_id) else {
                continue;
            };
            let matrix_name = if index == 0 {
                name.as_str()
            } else {
                object.name()
            };
            self.emit_matrix(state, object_id, object, world.to_array(), matrix_name)?;
        }
        for child_id in child_node_ids {
            self.emit_node(state, child_id, world)?;
        }

        Ok(())
    }

    /// Emits one matrix placing `object` at the world `position` and marks
    /// the object placed so the sweep skips it.
    fn emit_matrix<T>(
        &mut self,
        state: &VoxMain<T>,
        object_id: U32Id<BVoxObject>,
        object: &VoxObject,
        position: [i32; 3],
        name: &str,
    ) -> Result<()> {
        self.placed.insert(object_id.to_u32());

        let bounds = object.bounds();
        let [size_x, size_y, size_z] = bounds.to_array();
        let volume = size_x as usize * size_y as usize * size_z as usize;
        let mut voxels = vec![QbVoxel::default(); volume];

        let cell_color = resolve_cell_color_or_transparent(state, object)?;
        for voxel_id in object.iter_live() {
            let cell = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            // A Qubicle voxel stores no alpha, so the sampled color's alpha is
            // dropped.
            let [r, g, b, _] = cell_color.color(voxel_id);
            // Storage order: index = x + size_x * (y + size_y * z).
            let index = cell.x as usize
                + size_x as usize * (cell.y as usize + size_y as usize * cell.z as usize);
            voxels[index] = QbVoxel::new(r, g, b);
        }

        self.matrices.push(QbMatrix {
            name: name.to_owned(),
            size: [size_x, size_y, size_z],
            position,
            voxels,
        });

        Ok(())
    }
}
