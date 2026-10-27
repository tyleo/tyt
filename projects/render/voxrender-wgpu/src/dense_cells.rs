use branded_id::U32Id;
use voxrender::RenderObject;

/// The words the integer walk reads for `object`: its cells in raster order,
/// 0 when empty, else the material id plus one.
pub fn dense_cells(object: &RenderObject) -> Vec<u32> {
    let bounds = object.bounds();

    (0..bounds.x * bounds.y * bounds.z)
        .map(|index| {
            object
                .voxel_material(U32Id::from_u32(index))
                .map_or(0, |material_id| material_id.to_u32() + 1)
        })
        .collect()
}
