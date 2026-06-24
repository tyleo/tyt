use voxcore::VoxObject;
use voxj::VoxjCodecObject;

/// Builds a [`VoxjCodecObject`] from a [`VoxObject`], emitting one position and
/// sample row per live voxel in ascending raster order. Palette references map
/// back to palette indices (each id equals its index).
pub(crate) fn voxj_codec_object_from_vox_object(object: &VoxObject) -> VoxjCodecObject {
    let bounds = object.bounds();

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
        positions.push([position.x, position.y, position.z]);

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

    VoxjCodecObject {
        name: object.name().to_owned(),
        palette_refs,
        bounds: [bounds.x, bounds.y, bounds.z],
        positions,
        samples,
    }
}
