use voxcore::VoxObject;
use voxj::VoxjCodecObject;

/// Builds a [`VoxjCodecObject`] from a [`VoxObject`], emitting one position and
/// sample row per live voxel in ascending raster order. Palette references map
/// back to palette indices (each id equals its index).
pub(crate) fn voxj_codec_object_from_vox_object(object: &VoxObject) -> VoxjCodecObject {
    let bounds = [object.bounds.x, object.bounds.y, object.bounds.z];

    let palette_refs: Vec<usize> = object
        .palette_ref_ids
        .iter()
        .map(|id| unsafe { object.palette_refs.get(id) }.to_u32() as usize)
        .collect();

    let live_count = object.liveness.count_live();
    let mut positions = Vec::with_capacity(live_count);
    let mut samples = Vec::with_capacity(live_count);
    for voxel_id in object.iter_live() {
        let position = object.voxel_position(voxel_id);
        positions.push([position.x, position.y, position.z]);

        let row = object
            .palette_ref_ids
            .iter()
            .map(|palette_ref_id| {
                let column = unsafe { object.samples.get(palette_ref_id) };
                unsafe { column.get(voxel_id) }.to_u32()
            })
            .collect();
        samples.push(row);
    }

    VoxjCodecObject {
        name: object.name.clone(),
        palette_refs,
        bounds,
        positions,
        samples,
    }
}
