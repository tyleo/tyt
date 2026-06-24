use voxcore::{VoxMap, VoxValue};
use voxj::{VoxjMap, VoxjValue};

/// Converts a [`VoxjValue`] into the equivalent [`VoxValue`], recursing through
/// arrays and objects.
pub(crate) fn vox_value_from_voxj_value(value: &VoxjValue) -> VoxValue {
    match value {
        VoxjValue::Number(number) => VoxValue::Number(*number),
        VoxjValue::Text(text) => VoxValue::Text(text.clone()),
        VoxjValue::Bool(bool) => VoxValue::Bool(*bool),
        VoxjValue::Array(array) => {
            VoxValue::Array(array.iter().map(vox_value_from_voxj_value).collect())
        }
        VoxjValue::Object(VoxjMap(entries)) => VoxValue::Object(VoxMap(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), vox_value_from_voxj_value(value)))
                .collect(),
        )),
        VoxjValue::Null => VoxValue::Null,
    }
}
