use voxcore::VoxValuePoolValues;

/// The voxj type name of `values`'s kind.
pub fn value_pool_kind_name(values: VoxValuePoolValues<'_>) -> &'static str {
    match values {
        VoxValuePoolValues::Bool(_) => "bool",
        VoxValuePoolValues::Float(_) => "float",
        VoxValuePoolValues::Int(_) => "int",
        VoxValuePoolValues::Json(_) => "json",
        VoxValuePoolValues::String(_) => "string",
        VoxValuePoolValues::Vec2Float(_) => "vec-2-float",
        VoxValuePoolValues::Vec2Int(_) => "vec-2-int",
        VoxValuePoolValues::Vec3Float(_) => "vec-3-float",
        VoxValuePoolValues::Vec3Int(_) => "vec-3-int",
        VoxValuePoolValues::Vec4Float(_) => "vec-4-float",
        VoxValuePoolValues::Vec4Int(_) => "vec-4-int",
    }
}
