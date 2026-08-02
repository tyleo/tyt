use crate::{Result, vox_value_from_voxj_value};
use voxcore::VoxValuePool;
use voxj::VoxjValuePool;

/// Converts a [`VoxjValuePool`] into a [`VoxValuePool`], kind by kind. Every
/// kind maps one to one, carrying its values across unchanged; `json` values
/// recurse through [`vox_value_from_voxj_value`].
///
/// Errors if the value pool is empty or a value is outside its kind's value
/// domain.
pub fn vox_value_pool_from_voxj_value_pool(value_pool: &VoxjValuePool) -> Result<VoxValuePool> {
    Ok(match value_pool {
        VoxjValuePool::Bool(values) => VoxValuePool::boolean(values.clone())?,
        VoxjValuePool::Float(values) => VoxValuePool::float(values.clone())?,
        VoxjValuePool::Int(values) => VoxValuePool::int(values.clone())?,
        VoxjValuePool::Json(values) => VoxValuePool::json(
            values
                .iter()
                .map(vox_value_from_voxj_value)
                .collect::<Result<_>>()?,
        )?,
        VoxjValuePool::String(values) => VoxValuePool::string(values.clone())?,
        VoxjValuePool::Vec2Float(values) => VoxValuePool::vec_2_float(values.clone())?,
        VoxjValuePool::Vec2Int(values) => VoxValuePool::vec_2_int(values.clone())?,
        VoxjValuePool::Vec3Float(values) => VoxValuePool::vec_3_float(values.clone())?,
        VoxjValuePool::Vec3Int(values) => VoxValuePool::vec_3_int(values.clone())?,
        VoxjValuePool::Vec4Float(values) => VoxValuePool::vec_4_float(values.clone())?,
        VoxjValuePool::Vec4Int(values) => VoxValuePool::vec_4_int(values.clone())?,
    })
}
