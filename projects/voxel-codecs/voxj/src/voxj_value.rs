use crate::VoxjMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer, ser::Error as SerError};

/// An arbitrary Voxel Json value.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VoxjValue {
    /// A number, held as `f64` but serialized as a JSON integer when integral,
    /// so a value like `4` does not round-trip as `4.0`.
    Number(f64),

    /// A string.
    Text(String),

    /// A boolean.
    Bool(bool),

    /// An ordered list of values.
    Array(Vec<VoxjValue>),

    /// An ordered set of key/value pairs.
    Object(VoxjMap),

    /// JSON `null`.
    Null,
}

#[cfg(feature = "serde")]
impl Serialize for VoxjValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // JSON has no NaN or infinity; serde_json writes either as
            // `null`. A non-finite number errors instead of silently becoming
            // one.
            VoxjValue::Number(n) if !n.is_finite() => Err(SerError::custom(format!(
                "json value number must be finite, not {n}"
            ))),

            // Serialize an integral number as a JSON integer. The bound is
            // exclusive because `i64::MAX as f64` rounds up to `2^63`, and an
            // inclusive one would saturate the cast and write `2^63 - 1`.
            VoxjValue::Number(n)
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n < i64::MAX as f64 =>
            {
                serializer.serialize_i64(*n as i64)
            }

            VoxjValue::Number(n) => serializer.serialize_f64(*n),

            VoxjValue::Text(text) => serializer.serialize_str(text),

            VoxjValue::Bool(bool) => serializer.serialize_bool(*bool),

            VoxjValue::Array(array) => array.serialize(serializer),

            VoxjValue::Object(object) => object.serialize(serializer),

            VoxjValue::Null => serializer.serialize_unit(),
        }
    }
}

impl From<f64> for VoxjValue {
    fn from(v: f64) -> Self {
        VoxjValue::Number(v)
    }
}

impl From<String> for VoxjValue {
    fn from(v: String) -> Self {
        VoxjValue::Text(v)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::VoxjValue;

    #[test]
    fn non_finite_numbers_error_on_write() {
        // serde_json writes a non-finite number as `null`, which would destroy
        // the value the caller held.
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(serde_json::to_value(VoxjValue::Number(number)).is_err());
        }
    }

    #[test]
    fn the_integral_write_admits_only_what_the_cast_carries() {
        // `i64::MAX as f64` rounds up to `2^63`, which the cast cannot carry,
        // so it writes as a float rather than saturating to `2^63 - 1`.
        let above = i64::MAX as f64;
        assert_eq!(
            serde_json::to_string(&VoxjValue::Number(above)).unwrap(),
            "9.223372036854776e+18"
        );

        // `i64::MIN as f64` is exactly `-2^63` and casts losslessly.
        assert_eq!(
            serde_json::to_string(&VoxjValue::Number(i64::MIN as f64)).unwrap(),
            "-9223372036854775808"
        );
        assert_eq!(serde_json::to_string(&VoxjValue::Number(4.0)).unwrap(), "4");
    }
}
