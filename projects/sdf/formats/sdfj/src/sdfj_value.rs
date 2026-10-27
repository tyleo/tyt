use crate::SdfjMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer, ser::Error as SerError};

/// An arbitrary JSON value: what a `json` property holds.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SdfjValue {
    /// An ordered list of values.
    Array(Vec<SdfjValue>),

    /// A boolean.
    Bool(bool),

    /// JSON `null`.
    Null,

    /// A number. An integral number writes as a JSON integer: `4`, never
    /// `4.0`.
    Number(f64),

    /// An ordered set of key/value pairs.
    Object(SdfjMap),

    /// A string.
    Text(String),
}

#[cfg(feature = "serde")]
impl Serialize for SdfjValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SdfjValue::Array(array) => array.serialize(serializer),

            SdfjValue::Bool(bool) => serializer.serialize_bool(*bool),

            SdfjValue::Null => serializer.serialize_unit(),

            // JSON has no NaN or infinity, and serde_json writes either as
            // `null`. A non-finite number errors instead.
            SdfjValue::Number(number) if !number.is_finite() => Err(SerError::custom(format!(
                "json value number must be finite, not {number}"
            ))),

            // The bound is exclusive because `i64::MAX as f64` rounds up to
            // `2^63`, which the cast would saturate.
            SdfjValue::Number(number)
                if number.fract() == 0.0
                    && *number >= i64::MIN as f64
                    && *number < i64::MAX as f64 =>
            {
                serializer.serialize_i64(*number as i64)
            }

            SdfjValue::Number(number) => serializer.serialize_f64(*number),

            SdfjValue::Object(object) => object.serialize(serializer),

            SdfjValue::Text(text) => serializer.serialize_str(text),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::SdfjValue;

    #[test]
    fn non_finite_numbers_error_on_write() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(serde_json::to_value(SdfjValue::Number(number)).is_err());
        }
    }

    #[test]
    fn integral_numbers_write_as_integers() {
        assert_eq!(serde_json::to_string(&SdfjValue::Number(4.0)).unwrap(), "4");

        assert_eq!(
            serde_json::to_string(&SdfjValue::Number(i64::MAX as f64)).unwrap(),
            "9.223372036854776e+18"
        );
    }

    #[test]
    fn every_kind_round_trips() {
        let text = r#"[null,true,1.5,"s",{"k":[]}]"#;

        let value: SdfjValue = serde_json::from_str(text).unwrap();

        assert_eq!(serde_json::to_string(&value).unwrap(), text);
    }
}
