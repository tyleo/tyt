use crate::{Error, QbExtMatrix, QbVoxMain, Result};
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use voxcore::{VoxObject, color::resolve_cell_color_or_transparent};

/// Writes a [`QbVoxMain`] back to a decoded Qubicle Binary [`QbFile`],
/// the inverse of [`from_qb_file`](crate::from_qb_file).
///
/// Requires the `qb` ext the forward path writes; without it the file
/// cannot be rebuilt. Each object emits one matrix, taking its name, position,
/// and per-voxel visibility from the ext and its colors from the palette.
///
/// Errors if:
///
/// 1. the ext is missing
/// 2. its matrix entries do not line up with the objects
/// 3. a visibility list does not match its object
pub fn to_qb_file(state: &QbVoxMain) -> Result<QbFile> {
    let ext = match state.ext() {
        Some(ext) => ext.clone(),
        None => {
            return Err(Error::invalid(
                "state has no qb ext; cannot rebuild a Qubicle .qb file",
            ));
        }
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
fn matrix_from_object(
    state: &QbVoxMain,
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
