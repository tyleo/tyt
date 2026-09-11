use crate::{Error, Result, ext::QbExt};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use std::collections::HashSet;
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Writes a state to a Qubicle Binary file. [`QbExt`] rebuilds the loaded
/// file. `()` synthesizes one from the scene.
pub trait QbExtSource: VoxExt + Sized {
    /// Writes `state` to a [`QbFile`].
    fn write_qb(state: &VoxMain<Self>) -> Result<QbFile>;
}

impl QbExtSource for QbExt {
    /// Writes a state to a decoded Qubicle Binary [`QbFile`] through its ext,
    /// so a loaded file rebuilds exactly: each object emits one matrix, taking
    /// its name, position, and per-voxel visibility from the ext and its colors
    /// from the palette. An object retained after the load has no entry. It
    /// emits a matrix like a synthesized one at its first placement.
    ///
    /// Errors if:
    ///
    /// 1. the ext's matrix entries do not line up with the objects
    /// 2. a visibility list does not match its object
    /// 3. the header encodes visibility masks and an object has no entry
    /// 4. an object's `baseColor` draws from a non-color value pool
    fn write_qb(state: &VoxMain<Self>) -> Result<QbFile> {
        let ext = state.ext();

        let object_count = state.object_count();
        if object_count != ext.matrices.len() {
            return Err(Error::invalid(format!(
                "qb ext has {} matrices but the state has {object_count} objects",
                ext.matrices.len()
            )));
        }

        // The object is the author's build volume, so the written matrix keeps
        // its dimensions and voxel positions directly.
        let mut flattened = None;
        let matrices = state
            .iter_objects()
            .zip(&ext.matrices)
            .enumerate()
            .map(|(index, ((object_id, object), entry))| match entry {
                Some(provenance) => matrix_from_object(
                    state,
                    object,
                    &provenance.name,
                    provenance.position,
                    Some(&provenance.visibility),
                ),
                None => {
                    // A mask says which faces of a voxel show. Only the plain
                    // visibility byte has one solid value.
                    if ext.visibility_mask_encoded {
                        return Err(Error::invalid(format!(
                            "qb ext has no entry for object {index} and the header encodes visibility masks"
                        )));
                    }

                    let placement = flattened
                        .get_or_insert_with(|| placements(state))
                        .iter()
                        .find(|placement| placement.object_id == object_id)
                        .expect("every object has a placement");
                    matrix_from_object(state, object, &placement.name, placement.position, None)
                }
            })
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
}

impl QbExtSource for () {
    /// Synthesizes a Qubicle Binary file from the bare scene of a state
    /// carrying no `qb` ext, such as one cross-loaded from another format.
    ///
    /// Qubicle Binary has a flat matrix list and no hierarchy. `placements`
    /// flattens the scene. Each placement becomes one matrix. An object placed
    /// by several nodes is duplicated at each placement.
    ///
    /// Lossy only where Qubicle Binary cannot represent the source: grouping
    /// collapses, node rotation and scale are dropped, and a color's alpha is
    /// dropped because a Qubicle voxel stores none. The header is the default:
    /// `RGBA`, left-handed, uncompressed, with a plain solid visibility byte.
    fn write_qb(state: &VoxMain<Self>) -> Result<QbFile> {
        let matrices = placements(state)
            .iter()
            .map(|placement| {
                let object = state
                    .object(placement.object_id)
                    .expect("a placement's object is one of the state's");
                matrix_from_object(state, object, &placement.name, placement.position, None)
            })
            .collect::<Result<_>>()?;

        Ok(QbFile {
            matrices,
            ..QbFile::default()
        })
    }
}

/// Rebuilds a matrix grid from an object in `.qb` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer. `visibility`
/// supplies each live voxel's byte in raster order. Without it every solid
/// voxel takes the plain solid byte. Errors if the visibility count does not
/// match the object's solid voxels.
fn matrix_from_object<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    name: &str,
    position: [i32; 3],
    visibility: Option<&[u8]>,
) -> Result<QbMatrix> {
    let live_count = object.live_count();
    if let Some(visibility) = visibility
        && visibility.len() != live_count
    {
        return Err(Error::invalid(format!(
            "qb ext has {} visibility bytes but the object has {live_count} solid voxels",
            visibility.len()
        )));
    }

    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    for (live_index, voxel_id) in object.iter_live().enumerate() {
        let cell = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        let mut voxel = QbVoxel::new(r, g, b);
        if let Some(visibility) = visibility {
            voxel.visibility = visibility[live_index];
        }
        // Storage order: index = x + size_x * (y + size_y * z).
        let index = cell.x as usize
            + size_x as usize * (cell.y as usize + size_y as usize * cell.z as usize);
        voxels[index] = voxel;
    }

    Ok(QbMatrix {
        name: name.to_owned(),
        size: [size_x, size_y, size_z],
        position,
        voxels,
    })
}

/// One matrix of the flattened scene.
struct QbPlacement {
    object_id: U32Id<BVoxObject>,
    name: String,
    position: [i32; 3],
}

/// The matrices the scene flattens to, one per object placement in hierarchy
/// order. A placement lands at the world translation summed down from the
/// roots and rounded to whole voxels. An object no node places lands once at
/// the origin. A node's first object takes the node's name. The rest take
/// the object's name.
fn placements<T>(state: &VoxMain<T>) -> Vec<QbPlacement> {
    let mut placements = Vec::new();
    for &root_id in state.root_hierarchy_node_ids() {
        push_node_placements(state, root_id, TyVector3I32::new(0, 0, 0), &mut placements);
    }

    let placed: HashSet<U32Id<BVoxObject>> = placements
        .iter()
        .map(|placement| placement.object_id)
        .collect();
    for (object_id, object) in state.iter_objects() {
        if !placed.contains(&object_id) {
            placements.push(QbPlacement {
                object_id,
                name: object.name().to_owned(),
                position: [0, 0, 0],
            });
        }
    }

    placements
}

/// Walks `node_id` and its subtree. The translations sum into the world
/// position each placement carries.
fn push_node_placements<T>(
    state: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: TyVector3I32,
    placements: &mut Vec<QbPlacement>,
) {
    let node = state
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let world = parent + node.transform.position.round().as_ivec3();

    for (index, &object_id) in node.child_object_ids.iter().enumerate() {
        let object = state
            .object(object_id)
            .expect("a placed object is one of the state's");
        let name = if index == 0 {
            node.name.clone()
        } else {
            object.name().to_owned()
        };
        placements.push(QbPlacement {
            object_id,
            name,
            position: world.to_array(),
        });
    }

    for &child_id in &node.child_node_ids {
        push_node_placements(state, child_id, world, placements);
    }
}
