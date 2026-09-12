use branded_id::U32Id;
use voxcore::VoxObject;

/// A fresh object with `object`'s name, grid, origin, layers, and live
/// voxels, so a format that places one grid per object can give each extra
/// placement an object. A layer's default material, which only the empty
/// cells hold, is the material its first live voxel samples, or material `0`
/// for a layer with no live voxel.
pub fn duplicate_object(object: &VoxObject) -> VoxObject {
    let mut copy = VoxObject::new(object.name().to_owned(), object.bounds())
        .expect("the source object's grid is within the dense limit");
    copy.set_origin(object.origin());

    let layer_ids: Vec<_> = object.iter_layers().collect();
    let first_live = object.iter_live().next();
    for &(layer_id, palette_id) in &layer_ids {
        let default_material_id = first_live
            .and_then(|voxel_id| object.voxel_material(voxel_id, layer_id))
            .unwrap_or(U32Id::from_u32(0));
        copy.retain_layer(palette_id, default_material_id);
    }

    let mut sample_ids = Vec::with_capacity(layer_ids.len());
    for voxel_id in object.iter_live() {
        sample_ids.clear();
        sample_ids.extend(layer_ids.iter().map(|&(layer_id, _)| {
            object
                .voxel_material(voxel_id, layer_id)
                .expect("a live voxel samples every layer")
        }));
        copy.retain_voxel(voxel_id, &sample_ids)
            .expect("the copy has the source's grid and layers");
    }

    copy
}
