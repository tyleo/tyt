use crate::{Error, Result, vox_map_from_voxj_map};
use voxcore::VoxValue;
use voxj::VoxjValue;

/// Converts a [`VoxjValue`] into a [`VoxValue`], recursing into arrays and
/// objects. Rejects non-finite numbers; objects convert through
/// [`vox_map_from_voxj_map`].
pub fn vox_value_from_voxj_value(value: &VoxjValue) -> Result<VoxValue> {
    Ok(match value {
        VoxjValue::Number(number) => {
            if !number.is_finite() {
                return Err(Error::invalid(format!("number {number} must be finite")));
            }
            VoxValue::Number(*number)
        }
        VoxjValue::Text(text) => VoxValue::Text(text.clone()),
        VoxjValue::Bool(bool) => VoxValue::Bool(*bool),
        VoxjValue::Array(array) => VoxValue::Array(
            array
                .iter()
                .map(vox_value_from_voxj_value)
                .collect::<Result<_>>()?,
        ),
        VoxjValue::Object(object) => VoxValue::Object(vox_map_from_voxj_map(object)?),
        VoxjValue::Null => VoxValue::Null,
    })
}

#[cfg(test)]
mod tests {
    use crate::vox_value_from_voxj_value;
    use voxj::{VoxjMap, VoxjValue};

    #[test]
    fn rejects_non_finite_numbers() {
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(f64::NAN)).is_err());
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(f64::INFINITY)).is_err());
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(1.5)).is_ok());
    }

    /// A repeated key errors rather than resolving last-wins, which would
    /// silently drop the first value.
    #[test]
    fn rejects_a_repeated_object_key() {
        let value = VoxjValue::Object(VoxjMap(vec![
            ("k".to_owned(), VoxjValue::Number(1.0)),
            ("other".to_owned(), VoxjValue::Bool(true)),
            ("k".to_owned(), VoxjValue::Number(2.0)),
        ]));
        assert!(vox_value_from_voxj_value(&value).is_err());
    }
}
