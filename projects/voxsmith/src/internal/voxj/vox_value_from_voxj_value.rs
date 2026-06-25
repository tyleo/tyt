use crate::{Error, Result};
use voxcore::{VoxMap, VoxValue};
use voxj::{VoxjMap, VoxjValue};

/// Converts a [`VoxjValue`] into a [`VoxValue`], recursing into arrays and
/// objects. Rejects non-finite numbers and dedups duplicate object keys
/// last-wins (JSON convention).
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
        VoxjValue::Object(VoxjMap(entries)) => {
            let mut map: Vec<(String, VoxValue)> = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                let value = vox_value_from_voxj_value(value)?;
                // Last-wins: drop the earlier entry, append the new one.
                if let Some(existing) = map.iter().position(|(existing, _)| existing == key) {
                    map.remove(existing);
                }
                map.push((key.clone(), value));
            }
            VoxValue::Object(VoxMap(map))
        }
        VoxjValue::Null => VoxValue::Null,
    })
}

#[cfg(test)]
mod tests {
    use crate::vox_value_from_voxj_value;
    use voxcore::{VoxMap, VoxValue};
    use voxj::{VoxjMap, VoxjValue};

    #[test]
    fn rejects_non_finite_numbers() {
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(f64::NAN)).is_err());
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(f64::INFINITY)).is_err());
        assert!(vox_value_from_voxj_value(&VoxjValue::Number(1.5)).is_ok());
    }

    #[test]
    fn deduplicates_object_keys_last_wins() {
        let value = VoxjValue::Object(VoxjMap(vec![
            ("k".to_owned(), VoxjValue::Number(1.0)),
            ("other".to_owned(), VoxjValue::Bool(true)),
            ("k".to_owned(), VoxjValue::Number(2.0)),
        ]));
        let got = vox_value_from_voxj_value(&value).unwrap();
        assert_eq!(
            got,
            VoxValue::Object(VoxMap(vec![
                ("other".to_owned(), VoxValue::Bool(true)),
                ("k".to_owned(), VoxValue::Number(2.0)),
            ]))
        );
    }
}
