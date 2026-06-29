use voxcore::VoxObject;
use voxj_codec::VoxjDecodedObject;

/// Builds a [`VoxjDecodedObject`] from a [`VoxObject`], emitting the tight
/// runtime grid: one position and sample row per live voxel in ascending raster
/// order, rebased so the live voxels fill the grid from its origin. The
/// object's wider build volume, when it has margin, is recorded separately in
/// the document's edit state. Palette references map back to palette indices
/// (each id equals its index).
pub fn voxj_decoded_object_from_vox_object(object: &VoxObject) -> VoxjDecodedObject {
    let origin = object.origin();
    // The runtime grid is the live voxels' tight extent within the build
    // volume; an empty object collapses to a [0, 0, 0] grid at the build-volume
    // origin.
    let (min, size) = match object.live_extent() {
        Some((min, size)) => ([min.x, min.y, min.z], [size.x, size.y, size.z]),
        None => ([0, 0, 0], [0, 0, 0]),
    };

    // Reference ids, reused for each voxel's sample row.
    let palette_ref_ids: Vec<_> = object.iter_palette_refs().map(|(id, _)| id).collect();
    let palette_refs: Vec<usize> = object
        .iter_palette_refs()
        .map(|(_, palette)| palette.to_u32() as usize)
        .collect();

    let live_count = object.live_count();
    let mut positions = Vec::with_capacity(live_count);
    let mut samples = Vec::with_capacity(live_count);
    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel id is within the grid");
        positions.push([
            position.x - min[0],
            position.y - min[1],
            position.z - min[2],
        ]);

        let row = palette_ref_ids
            .iter()
            .map(|&palette_ref_id| {
                object
                    .voxel_cell(voxel_id, palette_ref_id)
                    .expect("a live voxel has a sample for every reference")
                    .to_u32()
            })
            .collect();
        samples.push(row);
    }

    VoxjDecodedObject {
        name: object.name().to_owned(),
        palette_refs,
        bounds: size,
        origin: [
            origin.x + min[0] as i32,
            origin.y + min[1] as i32,
            origin.z + min[2] as i32,
        ],
        positions,
        samples,
    }
}
