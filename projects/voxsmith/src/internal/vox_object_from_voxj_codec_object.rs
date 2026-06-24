use branded_id::U32Id;
use std::io;
use ty_math::TyVector3U32;
use voxcore::{BVoxPalette, BVoxPaletteCell, VoxObject};
use voxj::VoxjCodecObject;

/// Builds a [`VoxObject`] from a [`VoxjCodecObject`]: a dense grid whose
/// positions set liveness and samples.
///
/// Errors on an oversized grid, a position outside `bounds`, or ragged sample
/// rows. Cross-references are checked later by
/// [`VoxState::validate`](voxcore::VoxState::validate).
pub(crate) fn vox_object_from_voxj_codec_object(object: &VoxjCodecObject) -> io::Result<VoxObject> {
    let [size_x, size_y, size_z] = object.bounds;
    let bounds = TyVector3U32::new(size_x, size_y, size_z);

    let mut out = VoxObject::new(object.name.clone(), bounds).ok_or_else(|| {
        invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
            object.name,
            VoxObject::MAX_GRID_CELLS
        ))
    })?;

    // Back-fill cell 0 as a placeholder; live voxels overwrite theirs below.
    let filler = U32Id::<BVoxPaletteCell>::from_u32(0);
    for &palette_index in &object.palette_refs {
        out.add_palette_ref(U32Id::<BVoxPalette>::from_u32(palette_index as u32), filler);
    }

    if object.samples.len() != object.positions.len() {
        return Err(invalid(format!(
            "object \"{}\" has {} sample rows but {} positions",
            object.name,
            object.samples.len(),
            object.positions.len()
        )));
    }

    for (&[x, y, z], row) in object.positions.iter().zip(&object.samples) {
        let voxel_id = out.voxel_id(TyVector3U32::new(x, y, z)).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" position [{x}, {y}, {z}] lies outside bounds [{size_x}, {size_y}, {size_z}]",
                object.name
            ))
        })?;

        let cells: Vec<U32Id<BVoxPaletteCell>> =
            row.iter().map(|&cell| U32Id::from_u32(cell)).collect();
        out.retain_voxel(voxel_id, &cells).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" sample row at [{x}, {y}, {z}] has {} values but references {} palettes",
                object.name,
                row.len(),
                object.palette_refs.len()
            ))
        })?;
    }

    Ok(out)
}

/// Invalid-data error from a message.
fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
