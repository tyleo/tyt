use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{BVoxValuePoolValue, VoxMain, VoxValuePool};

/// The value pool and value id of `attribute` on the material the voxel at
/// `position` samples, in the first object's first layer.
pub(crate) fn voxel_attribute<'a>(
    main: &'a VoxMain,
    position: TyVector3U32,
    attribute: &str,
) -> (&'a VoxValuePool, U32Id<BVoxValuePoolValue>) {
    let (_, object) = main.iter_objects().next().unwrap();
    let voxel_id = object.voxel_id(position).unwrap();
    let (layer_id, palette_id) = object.iter_layers().next().unwrap();
    let material_id = object.voxel_material(voxel_id, layer_id).unwrap();
    let palette = main.palette(palette_id).unwrap();
    let property_id = palette.property_id_by_name(attribute).unwrap();
    main.material_value(palette_id, material_id, property_id)
        .unwrap()
}
