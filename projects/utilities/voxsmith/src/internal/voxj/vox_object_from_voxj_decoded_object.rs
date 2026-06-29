use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxPalette, BVoxPaletteCell, VoxObject};
use voxj_codec::VoxjDecodedObject;

/// Builds a [`VoxObject`] from a [`VoxjDecodedObject`] and its optional build
/// volume.
///
/// The decoded object holds the tight runtime grid. When `edit` is `Some`, the
/// object is built in that build volume (the author's edit grid) with each voxel
/// shifted from the runtime grid into it, recovering the margin the document
/// recorded. When `edit` is `None`, the build volume equals the tight grid.
///
/// Errors on an oversized grid, a position outside the grid, or ragged sample
/// rows. Cross-references are checked later by
/// [`VoxMain::validate`](voxcore::VoxMain::validate).
pub fn vox_object_from_voxj_decoded_object(
    object: &VoxjDecodedObject,
    edit: Option<([u32; 3], [i32; 3])>,
) -> Result<VoxObject> {
    // The grid the voxcore object lives in (its build volume), its placing origin,
    // and the offset shifting a runtime-grid position into it.
    let (bounds, origin, offset) = match edit {
        Some((bounds, origin)) => (
            bounds,
            origin,
            [
                object.origin[0] - origin[0],
                object.origin[1] - origin[1],
                object.origin[2] - origin[2],
            ],
        ),
        None => (object.bounds, object.origin, [0, 0, 0]),
    };
    let [size_x, size_y, size_z] = bounds;

    let mut out = VoxObject::new(
        object.name.clone(),
        TyVector3U32::new(size_x, size_y, size_z),
    )
    .ok_or_else(|| {
        invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
            object.name,
            VoxObject::MAX_GRID_CELLS
        ))
    })?;

    out.set_origin(TyVector3I32::new(origin[0], origin[1], origin[2]));

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
        let shifted = [
            x as i64 + offset[0] as i64,
            y as i64 + offset[1] as i64,
            z as i64 + offset[2] as i64,
        ];
        let position = in_bounds(shifted, bounds).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" position [{x}, {y}, {z}] lies outside its grid [{size_x}, {size_y}, {size_z}]",
                object.name
            ))
        })?;
        // `in_bounds` already confirmed the position fits the grid.
        let voxel_id = out.voxel_id(position).expect("position is within bounds");

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

/// The `[x, y, z]` point as a grid position, or `None` if any axis is negative or
/// reaches `bounds`.
fn in_bounds(p: [i64; 3], bounds: [u32; 3]) -> Option<TyVector3U32> {
    let inside = (0..3).all(|a| p[a] >= 0 && p[a] < bounds[a] as i64);
    inside.then(|| TyVector3U32::new(p[0] as u32, p[1] as u32, p[2] as u32))
}

/// Invalid-data error from a message.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}
