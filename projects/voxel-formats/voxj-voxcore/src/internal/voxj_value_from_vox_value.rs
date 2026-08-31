use crate::voxj_map_from_vox_map;
use voxcore::VoxValue;
use voxj::VoxjValue;

/// Converts a [`VoxValue`] into the equivalent [`VoxjValue`], recursing through
/// arrays and objects.
pub fn voxj_value_from_vox_value(value: &VoxValue) -> VoxjValue {
    match value {
        VoxValue::Number(number) => VoxjValue::Number(*number),
        VoxValue::Text(text) => VoxjValue::Text(text.clone()),
        VoxValue::Bool(bool) => VoxjValue::Bool(*bool),
        VoxValue::Array(array) => {
            VoxjValue::Array(array.iter().map(voxj_value_from_vox_value).collect())
        }
        VoxValue::Object(object) => VoxjValue::Object(voxj_map_from_vox_map(object)),
        VoxValue::Null => VoxjValue::Null,
    }
}
