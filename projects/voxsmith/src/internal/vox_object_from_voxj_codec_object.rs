use branded_id::{U32Id, soa::IdField};
use std::io;
use ty_math::TyVector3U32;
use voxcore::{BVoxPalette, BVoxPaletteCell, BVoxPaletteRef, BVoxVoxel, VoxLiveness, VoxObject};
use voxj::{VoxjCodecObject, VoxjPalette};

/// The largest dense voxel grid a single object may materialize, as a cell
/// count. The grid allocates storage for every cell regardless of how many are
/// live (roughly a dozen bytes per cell), so without a ceiling a small but
/// sparse `.voxj` with huge `bounds` would force a multi-gigabyte allocation.
/// This bounds an object to a few hundred megabytes; raise it only if larger
/// dense objects are genuinely needed. Always at most `u32::MAX`, so a voxel id
/// is always a valid raster index.
pub(crate) const MAX_GRID_CELLS: u64 = 1 << 24;

/// Builds a [`VoxObject`] from a [`VoxjCodecObject`] as a dense raster grid: a
/// voxel id is allocated for every cell of `bounds`, with id equal to the raster
/// index `x * Y * Z + y * Z + z`, and the object's positions set the matching
/// liveness bits and samples. `palettes` is the document's palette list, used to
/// validate that voxel samples reference cells that exist.
///
/// Errors if the grid exceeds [`MAX_GRID_CELLS`], a position lies outside
/// `bounds`, a palette reference or sample cell is out of range, or the sample
/// rows are ragged.
pub(crate) fn vox_object_from_voxj_codec_object(
    object: &VoxjCodecObject,
    palettes: &[VoxjPalette],
) -> io::Result<VoxObject> {
    let [size_x, size_y, size_z] = object.bounds;
    let volume = size_x as u64 * size_y as u64 * size_z as u64;
    if volume > MAX_GRID_CELLS {
        return Err(invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} = {volume} cells exceeds the dense limit of {MAX_GRID_CELLS} cells",
            object.name
        )));
    }
    let volume = volume as usize;

    validate_object(object, palettes)?;

    let mut out = VoxObject::default();
    out.name = object.name.clone();
    out.bounds = TyVector3U32::new(size_x, size_y, size_z);
    out.liveness = VoxLiveness::new(volume);

    // Allocate a voxel id for every grid cell so the columns stay dense; the
    // pool hands out ids 0, 1, ... in order, so each equals its raster index.
    for _ in 0..volume {
        out.voxel_ids.retain();
    }

    // One sample column per palette reference, filled so every voxel id has a
    // value (branded_id requires it). Live voxels overwrite theirs below; the
    // filler in empty cells is never read back (the reverse walks only live
    // voxels), so cell 0 is a fine placeholder even for an empty palette.
    for &palette_index in &object.palette_refs {
        let palette_ref_id = out.palette_ref_ids.retain();
        out.palette_refs.retain(
            palette_ref_id,
            U32Id::<BVoxPalette>::from_u32(palette_index as u32),
        );

        let filler = U32Id::<BVoxPaletteCell>::from_u32(0);
        let mut column = IdField::new();
        for voxel_id in &out.voxel_ids {
            column.retain(voxel_id, filler);
        }
        out.samples.retain(palette_ref_id, column);
    }

    // Mark live cells and write their samples.
    for (voxel_index, &[x, y, z]) in object.positions.iter().enumerate() {
        let raster = x as u64 * size_y as u64 * size_z as u64 + y as u64 * size_z as u64 + z as u64;
        let voxel_id = U32Id::<BVoxVoxel>::from_u32(raster as u32);
        out.liveness.set_live(voxel_id, true);

        for (palette_ref_index, &cell) in object.samples[voxel_index].iter().enumerate() {
            let palette_ref_id = U32Id::<BVoxPaletteRef>::from_u32(palette_ref_index as u32);
            let column = unsafe { out.samples.get_mut(palette_ref_id) };
            unsafe { column.set(voxel_id, U32Id::<BVoxPaletteCell>::from_u32(cell)) };
        }
    }

    Ok(out)
}

/// Validates the object's references and sample shape so building can index
/// positions and samples directly.
fn validate_object(object: &VoxjCodecObject, palettes: &[VoxjPalette]) -> io::Result<()> {
    let [size_x, size_y, size_z] = object.bounds;
    let palette_count = object.palette_refs.len();

    // Every reference must point at a real palette. A reference to an empty
    // palette is fine on its own: it only matters if a voxel samples it, which
    // the per-sample range check below rejects.
    for (palette_ref_index, &palette_index) in object.palette_refs.iter().enumerate() {
        if palette_index >= palettes.len() {
            return Err(invalid(format!(
                "object \"{}\" palette ref {palette_ref_index} -> {palette_index} is out of range of {} palettes",
                object.name,
                palettes.len()
            )));
        }
    }

    if object.samples.len() != object.positions.len() {
        return Err(invalid(format!(
            "object \"{}\" has {} sample rows but {} positions",
            object.name,
            object.samples.len(),
            object.positions.len()
        )));
    }

    for (voxel_index, &[x, y, z]) in object.positions.iter().enumerate() {
        if x >= size_x || y >= size_y || z >= size_z {
            return Err(invalid(format!(
                "object \"{}\" position [{x}, {y}, {z}] lies outside bounds [{size_x}, {size_y}, {size_z}]",
                object.name
            )));
        }

        let row = &object.samples[voxel_index];
        if row.len() != palette_count {
            return Err(invalid(format!(
                "object \"{}\" sample row {voxel_index} has {} values but references {palette_count} palettes",
                object.name,
                row.len()
            )));
        }
        for (palette_ref_index, &cell) in row.iter().enumerate() {
            let cells = palettes[object.palette_refs[palette_ref_index]].data.len();
            if cell as usize >= cells {
                return Err(invalid(format!(
                    "object \"{}\" sample [{voxel_index}][{palette_ref_index}] = {cell} is out of range of {cells} cells",
                    object.name
                )));
            }
        }
    }

    Ok(())
}

/// Wraps a message describing malformed input as invalid data.
fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
