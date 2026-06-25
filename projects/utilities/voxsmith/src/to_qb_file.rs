use crate::{Error, QubicleQbExtWrapper, QubicleQbMatrix, Result, from_vox_value};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use voxcore::{
    BVoxAttribute, BVoxPaletteRef, BVoxVoxel, VoxObject, VoxPalette, VoxState, VoxValue,
};

/// Writes a [`VoxState`] back to a decoded Qubicle Binary [`QbFile`], the inverse
/// of [`from_qb_file`](crate::from_qb_file).
///
/// Requires the `qubicle-qb` ext the forward path writes; without it the file
/// cannot be rebuilt. Each object emits one matrix, taking its name, position, and
/// per-voxel visibility from the ext and its colors from the palette.
///
/// Errors if the ext is missing, its matrix entries do not line up with the
/// objects, or a visibility list does not match its object.
pub fn to_qb_file(state: &VoxState) -> Result<QbFile> {
    let ext = match state.ext() {
        Some(ext) => from_vox_value::<QubicleQbExtWrapper>(ext)?.qubicle_qb,
        None => {
            return Err(Error::invalid(
                "state has no qubicle-qb ext; cannot rebuild a Qubicle .qb file",
            ));
        }
    };

    let object_count = state.object_count();
    if object_count != ext.matrices.len() {
        return Err(Error::invalid(format!(
            "qubicle-qb ext has {} matrices but the state has {object_count} objects",
            ext.matrices.len()
        )));
    }

    let palette = state.iter_palettes().next().map(|(_, palette)| palette);
    let matrices = state
        .iter_objects()
        .zip(&ext.matrices)
        .map(|((_, object), provenance)| matrix_from_object(object, palette, provenance))
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

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from the
/// palette and its visibility from the aligned ext list, placed in `.qb` storage
/// order. Errors if the visibility count does not match the object's solid voxels.
fn matrix_from_object(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    provenance: &QubicleQbMatrix,
) -> Result<QbMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = [bounds.x, bounds.y, bounds.z];
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbVoxel::default(); volume];

    let reference = object.iter_palette_refs().next().map(|(id, _)| id);
    let live: Vec<_> = object.iter_live().collect();
    if live.len() != provenance.visibility.len() {
        return Err(Error::invalid(format!(
            "qubicle-qb ext has {} visibility bytes but the object has {} solid voxels",
            provenance.visibility.len(),
            live.len()
        )));
    }

    for (&voxel, &visibility) in live.iter().zip(&provenance.visibility) {
        let position = object
            .voxel_position(voxel)
            .expect("a live voxel is within the grid");
        let [r, g, b] = voxel_color(object, palette, reference, voxel);
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
