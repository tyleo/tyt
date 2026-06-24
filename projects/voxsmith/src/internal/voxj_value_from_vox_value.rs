use voxcore::{VoxMap, VoxValue};
use voxj::{VoxjMap, VoxjValue};

/// Converts a [`VoxValue`] into the equivalent [`VoxjValue`], recursing through
/// arrays and objects.
pub(crate) fn voxj_value_from_vox_value(value: &VoxValue) -> VoxjValue {
    match value {
        VoxValue::Number(number) => VoxjValue::Number(*number),
        VoxValue::Text(text) => VoxjValue::Text(text.clone()),
        VoxValue::Bool(bool) => VoxjValue::Bool(*bool),
        VoxValue::Array(array) => {
            VoxjValue::Array(array.iter().map(voxj_value_from_vox_value).collect())
        }
        VoxValue::Object(VoxMap(entries)) => VoxjValue::Object(VoxjMap(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), voxj_value_from_vox_value(value)))
                .collect(),
        )),
        VoxValue::Null => VoxjValue::Null,
    }
}
