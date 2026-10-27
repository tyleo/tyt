use crate::FindNonFinite;
use serde::{Serialize, Serializer, ser::Error as SerError};

/// Serializes a field whose numbers are all finite. A NaN or an infinity
/// errors because serde_json would write either as `null`.
pub fn finite<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: FindNonFinite + Serialize,
    S: Serializer,
{
    if let Some(number) = value.find_non_finite() {
        return Err(SerError::custom(format!(
            "number must be finite, not {number}"
        )));
    }

    value.serialize(serializer)
}
