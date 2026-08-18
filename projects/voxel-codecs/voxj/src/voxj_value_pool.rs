use crate::VoxjValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A shared value pool, its values all of one value-shape. The variant is the
/// value pool's kind and its payload is the `values`. Palettes reference value
/// pools by index and read values out by value-index.
///
/// Serde tags the variant by a `kind` field and its payload by a `values`
/// field, so every kind's wire shape is `{ "kind", "values" }`. Value shapes
/// are typed per kind, so a malformed value, an unrecognized kind, and an
/// unknown key all reject at parse.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        content = "values",
        rename_all = "kebab-case",
        deny_unknown_fields
    )
)]
pub enum VoxjValuePool {
    /// Boolean values.
    Bool(Vec<bool>),

    /// Float values: finite numbers, with the infinities spelled by the
    /// `"inf"` and `"-inf"` sentinels.
    Float(#[cfg_attr(feature = "serde", serde(with = "values::float"))] Vec<f64>),

    /// Int values: integer-spelled, magnitude at most `2^53 - 1`.
    Int(#[cfg_attr(feature = "serde", serde(with = "values::int"))] Vec<i64>),

    /// Arbitrary JSON values, including `null`.
    Json(Vec<VoxjValue>),

    /// String values.
    String(Vec<String>),

    /// Two-component float vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-2-float"))]
    Vec2Float(#[cfg_attr(feature = "serde", serde(with = "values::float::array"))] Vec<[f64; 2]>),

    /// Two-component int vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-2-int"))]
    Vec2Int(#[cfg_attr(feature = "serde", serde(with = "values::int::array"))] Vec<[i64; 2]>),

    /// Three-component float vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-3-float"))]
    Vec3Float(#[cfg_attr(feature = "serde", serde(with = "values::float::array"))] Vec<[f64; 3]>),

    /// Three-component int vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-3-int"))]
    Vec3Int(#[cfg_attr(feature = "serde", serde(with = "values::int::array"))] Vec<[i64; 3]>),

    /// Four-component float vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-4-float"))]
    Vec4Float(#[cfg_attr(feature = "serde", serde(with = "values::float::array"))] Vec<[f64; 4]>),

    /// Four-component int vectors.
    #[cfg_attr(feature = "serde", serde(rename = "vec-4-int"))]
    Vec4Int(#[cfg_attr(feature = "serde", serde(with = "values::int::array"))] Vec<[i64; 4]>),
}

impl VoxjValuePool {
    /// The number of values in the value pool, across every kind. A palette's
    /// value-indices must fall in `[0, len)`.
    pub fn len(&self) -> usize {
        match self {
            VoxjValuePool::Bool(values) => values.len(),
            VoxjValuePool::Float(values) => values.len(),
            VoxjValuePool::Int(values) => values.len(),
            VoxjValuePool::Json(values) => values.len(),
            VoxjValuePool::String(values) => values.len(),
            VoxjValuePool::Vec2Float(values) => values.len(),
            VoxjValuePool::Vec2Int(values) => values.len(),
            VoxjValuePool::Vec3Float(values) => values.len(),
            VoxjValuePool::Vec3Int(values) => values.len(),
            VoxjValuePool::Vec4Float(values) => values.len(),
            VoxjValuePool::Vec4Int(values) => values.len(),
        }
    }

    /// Whether the value pool holds no values.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Serde for the numeric `values` payloads.
#[cfg(feature = "serde")]
mod values {
    /// The `float` payload forms.
    pub mod float {
        use serde::{
            Deserialize, Deserializer, Serialize, Serializer,
            de::{Error as DeError, Unexpected, Visitor},
            ser::Error as SerError,
        };
        use std::fmt::{Formatter, Result as FmtResult};

        /// Serializes a `float` value pool payload: an infinity as its
        /// `"inf"` or `"-inf"` sentinel, an integral number as a JSON integer
        /// so `1` does not round-trip as `1.0`, and NaN as an error because
        /// JSON cannot spell it.
        pub fn serialize<S>(values: &[f64], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_seq(values.iter().map(|&value| FloatValue(value)))
        }

        /// Deserializes a `float` value pool payload: each value a finite
        /// number, `"inf"`, or `"-inf"`.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let values = Vec::<FloatValue>::deserialize(deserializer)?;
            Ok(values.into_iter().map(|FloatValue(value)| value).collect())
        }

        /// One float value on the wire.
        struct FloatValue(f64);

        impl Serialize for FloatValue {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let value = self.0;
                if value.is_nan() {
                    Err(SerError::custom("float value must not be NaN"))
                } else if value == f64::INFINITY {
                    serializer.serialize_str("inf")
                } else if value == f64::NEG_INFINITY {
                    serializer.serialize_str("-inf")
                } else if value.fract() == 0.0
                    && value >= i64::MIN as f64
                    && value < i64::MAX as f64
                {
                    // `i64::MAX as f64` rounds up to `2^63`, so the exclusive
                    // bound admits exactly the integral values an `as` cast
                    // converts losslessly.
                    serializer.serialize_i64(value as i64)
                } else {
                    serializer.serialize_f64(value)
                }
            }
        }

        impl<'de> Deserialize<'de> for FloatValue {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(FloatValueVisitor)
            }
        }

        struct FloatValueVisitor;

        impl Visitor<'_> for FloatValueVisitor {
            type Value = FloatValue;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("a finite number, \"inf\", or \"-inf\"")
            }

            fn visit_i64<E>(self, value: i64) -> Result<FloatValue, E>
            where
                E: DeError,
            {
                Ok(FloatValue(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<FloatValue, E>
            where
                E: DeError,
            {
                Ok(FloatValue(value as f64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<FloatValue, E>
            where
                E: DeError,
            {
                if value.is_finite() {
                    Ok(FloatValue(value))
                } else {
                    Err(E::invalid_value(Unexpected::Float(value), &self))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<FloatValue, E>
            where
                E: DeError,
            {
                match value {
                    "inf" => Ok(FloatValue(f64::INFINITY)),
                    "-inf" => Ok(FloatValue(f64::NEG_INFINITY)),
                    _ => Err(E::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        /// The `vec-*-float` payload forms: each value an array of exactly
        /// `N` float values.
        pub mod array {
            use crate::voxj_value_pool::values::float::FloatValue;
            use serde::{
                Deserialize, Deserializer, Serialize, Serializer,
                de::{Error as DeError, IgnoredAny, SeqAccess, Visitor},
                ser::SerializeTuple,
            };
            use std::fmt::{Formatter, Result as FmtResult};

            /// Serializes a `vec-*-float` value pool payload.
            pub fn serialize<S, const N: usize>(
                values: &[[f64; N]],
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(values.iter().map(|&value| FloatArray(value)))
            }

            /// Deserializes a `vec-*-float` value pool payload.
            pub fn deserialize<'de, D, const N: usize>(
                deserializer: D,
            ) -> Result<Vec<[f64; N]>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let values = Vec::<FloatArray<N>>::deserialize(deserializer)?;
                Ok(values.into_iter().map(|FloatArray(value)| value).collect())
            }

            /// One float vector on the wire.
            struct FloatArray<const N: usize>([f64; N]);

            impl<const N: usize> Serialize for FloatArray<N> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = serializer.serialize_tuple(N)?;
                    for &component in &self.0 {
                        tuple.serialize_element(&FloatValue(component))?;
                    }
                    tuple.end()
                }
            }

            impl<'de, const N: usize> Deserialize<'de> for FloatArray<N> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_tuple(N, FloatArrayVisitor)
                }
            }

            struct FloatArrayVisitor<const N: usize>;

            impl<'de, const N: usize> Visitor<'de> for FloatArrayVisitor<N> {
                type Value = FloatArray<N>;

                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    write!(formatter, "an array of exactly {N} float values")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<FloatArray<N>, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut values = [0.0; N];
                    for (index, slot) in values.iter_mut().enumerate() {
                        let Some(FloatValue(value)) = seq.next_element()? else {
                            return Err(A::Error::invalid_length(index, &self));
                        };
                        *slot = value;
                    }
                    if seq.next_element::<IgnoredAny>()?.is_some() {
                        return Err(A::Error::invalid_length(N + 1, &self));
                    }
                    Ok(FloatArray(values))
                }
            }
        }
    }

    /// The `int` payload forms.
    pub mod int {
        use serde::{
            Deserialize, Deserializer, Serialize, Serializer,
            de::{Error as DeError, Unexpected, Visitor},
            ser::Error as SerError,
        };
        use std::fmt::{Formatter, Result as FmtResult};

        /// The largest magnitude an int value may carry: `2^53 - 1`, the
        /// widest span a consumer reading numbers as doubles keeps exact.
        const MAX_MAGNITUDE: i64 = (1 << 53) - 1;

        /// Serializes an `int` value pool payload, erroring on a value beyond
        /// `2^53 - 1` in magnitude.
        pub fn serialize<S>(values: &[i64], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.collect_seq(values.iter().map(|&value| IntValue(value)))
        }

        /// Deserializes an `int` value pool payload: each value
        /// integer-spelled with magnitude at most `2^53 - 1`.
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let values = Vec::<IntValue>::deserialize(deserializer)?;
            Ok(values.into_iter().map(|IntValue(value)| value).collect())
        }

        /// One int value on the wire.
        struct IntValue(i64);

        impl Serialize for IntValue {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                if (-MAX_MAGNITUDE..=MAX_MAGNITUDE).contains(&self.0) {
                    serializer.serialize_i64(self.0)
                } else {
                    Err(SerError::custom(
                        "int value magnitude must be at most 2^53 - 1",
                    ))
                }
            }
        }

        impl<'de> Deserialize<'de> for IntValue {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(IntValueVisitor)
            }
        }

        /// Reads a JSON number spelled as an integer; a fractional or
        /// exponent spelling arrives as a float and rejects.
        struct IntValueVisitor;

        impl Visitor<'_> for IntValueVisitor {
            type Value = IntValue;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("an integer with magnitude at most 2^53 - 1")
            }

            fn visit_i64<E>(self, value: i64) -> Result<IntValue, E>
            where
                E: DeError,
            {
                if (-MAX_MAGNITUDE..=MAX_MAGNITUDE).contains(&value) {
                    Ok(IntValue(value))
                } else {
                    Err(E::invalid_value(Unexpected::Signed(value), &self))
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<IntValue, E>
            where
                E: DeError,
            {
                if value <= MAX_MAGNITUDE as u64 {
                    Ok(IntValue(value as i64))
                } else {
                    Err(E::invalid_value(Unexpected::Unsigned(value), &self))
                }
            }

            fn visit_f64<E>(self, value: f64) -> Result<IntValue, E>
            where
                E: DeError,
            {
                Err(E::invalid_value(Unexpected::Float(value), &self))
            }
        }

        /// The `vec-*-int` payload forms: each value an array of exactly `N`
        /// int values.
        pub mod array {
            use crate::voxj_value_pool::values::int::IntValue;
            use serde::{
                Deserialize, Deserializer, Serialize, Serializer,
                de::{Error as DeError, IgnoredAny, SeqAccess, Visitor},
                ser::SerializeTuple,
            };
            use std::fmt::{Formatter, Result as FmtResult};

            /// Serializes a `vec-*-int` value pool payload.
            pub fn serialize<S, const N: usize>(
                values: &[[i64; N]],
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_seq(values.iter().map(|&value| IntArray(value)))
            }

            /// Deserializes a `vec-*-int` value pool payload.
            pub fn deserialize<'de, D, const N: usize>(
                deserializer: D,
            ) -> Result<Vec<[i64; N]>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let values = Vec::<IntArray<N>>::deserialize(deserializer)?;
                Ok(values.into_iter().map(|IntArray(value)| value).collect())
            }

            /// One int vector on the wire.
            struct IntArray<const N: usize>([i64; N]);

            impl<const N: usize> Serialize for IntArray<N> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let mut tuple = serializer.serialize_tuple(N)?;
                    for &component in &self.0 {
                        tuple.serialize_element(&IntValue(component))?;
                    }
                    tuple.end()
                }
            }

            impl<'de, const N: usize> Deserialize<'de> for IntArray<N> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_tuple(N, IntArrayVisitor)
                }
            }

            struct IntArrayVisitor<const N: usize>;

            impl<'de, const N: usize> Visitor<'de> for IntArrayVisitor<N> {
                type Value = IntArray<N>;

                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    write!(formatter, "an array of exactly {N} int values")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<IntArray<N>, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut values = [0; N];
                    for (index, slot) in values.iter_mut().enumerate() {
                        let Some(IntValue(value)) = seq.next_element()? else {
                            return Err(A::Error::invalid_length(index, &self));
                        };
                        *slot = value;
                    }
                    if seq.next_element::<IgnoredAny>()?.is_some() {
                        return Err(A::Error::invalid_length(N + 1, &self));
                    }
                    Ok(IntArray(values))
                }
            }
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{VoxjValue, VoxjValuePool};
    use serde_json::{Value, json};

    #[test]
    fn every_kind_round_trips_the_wire_shape() {
        for (value_pool, wire) in [
            (
                VoxjValuePool::Bool(vec![true, false]),
                json!({ "kind": "bool", "values": [true, false] }),
            ),
            (
                VoxjValuePool::Float(vec![0.0, 0.5]),
                json!({ "kind": "float", "values": [0, 0.5] }),
            ),
            (
                VoxjValuePool::Int(vec![0, 128]),
                json!({ "kind": "int", "values": [0, 128] }),
            ),
            (
                VoxjValuePool::Json(vec![VoxjValue::Null, VoxjValue::Number(3.0)]),
                json!({ "kind": "json", "values": [null, 3] }),
            ),
            (
                VoxjValuePool::String(vec!["low".to_owned(), "high".to_owned()]),
                json!({ "kind": "string", "values": ["low", "high"] }),
            ),
            (
                VoxjValuePool::Vec2Float(vec![[0.25, 0.75]]),
                json!({ "kind": "vec-2-float", "values": [[0.25, 0.75]] }),
            ),
            (
                VoxjValuePool::Vec2Int(vec![[3, 7]]),
                json!({ "kind": "vec-2-int", "values": [[3, 7]] }),
            ),
            (
                VoxjValuePool::Vec3Float(vec![[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]]),
                json!({ "kind": "vec-3-float", "values": [[0, 0, 1], [1, 0, 0]] }),
            ),
            (
                VoxjValuePool::Vec3Int(vec![[3, 7, 1]]),
                json!({ "kind": "vec-3-int", "values": [[3, 7, 1]] }),
            ),
            (
                VoxjValuePool::Vec4Float(vec![[1.0, 0.0, 0.0, 1.0]]),
                json!({ "kind": "vec-4-float", "values": [[1, 0, 0, 1]] }),
            ),
            (
                VoxjValuePool::Vec4Int(vec![[3, 7, 1, 0]]),
                json!({ "kind": "vec-4-int", "values": [[3, 7, 1, 0]] }),
            ),
        ] {
            assert_eq!(serde_json::to_value(&value_pool).unwrap(), wire);
            assert_eq!(
                serde_json::from_value::<VoxjValuePool>(wire).unwrap(),
                value_pool
            );
        }
    }

    #[test]
    fn float_sentinels_round_trip() {
        for (value_pool, wire) in [
            (
                VoxjValuePool::Float(vec![f64::INFINITY, f64::NEG_INFINITY, 0.5]),
                json!({ "kind": "float", "values": ["inf", "-inf", 0.5] }),
            ),
            (
                VoxjValuePool::Vec3Float(vec![[0.0, f64::INFINITY, f64::NEG_INFINITY]]),
                json!({ "kind": "vec-3-float", "values": [[0, "inf", "-inf"]] }),
            ),
        ] {
            assert_eq!(serde_json::to_value(&value_pool).unwrap(), wire);
            assert_eq!(
                serde_json::from_value::<VoxjValuePool>(wire).unwrap(),
                value_pool
            );
        }
    }

    #[test]
    fn values_first_wire_order_parses() {
        let value_pool: VoxjValuePool =
            serde_json::from_str(r#"{"values":["inf",2.5],"kind":"float"}"#).unwrap();
        assert_eq!(value_pool, VoxjValuePool::Float(vec![f64::INFINITY, 2.5]));
    }

    #[test]
    fn nan_errors_on_write() {
        assert!(serde_json::to_value(VoxjValuePool::Float(vec![f64::NAN])).is_err());
        assert!(serde_json::to_value(VoxjValuePool::Vec2Float(vec![[0.0, f64::NAN]])).is_err());
    }

    #[test]
    fn integral_floats_write_as_integers() {
        let text = serde_json::to_string(&VoxjValuePool::Float(vec![1.0, 2.5])).unwrap();
        assert_eq!(text, r#"{"kind":"float","values":[1,2.5]}"#);
    }

    // These three values mis-parse by one ULP without serde_json's
    // float_roundtrip feature, so this round trip proves the manifest
    // carries it.
    #[test]
    fn seventeen_digit_floats_round_trip_through_text() {
        let text = concat!(
            r#"{"kind":"float","values":"#,
            "[0.0009105809506465125,0.21586050011389926,0.9734452903984125]}",
        );
        let value_pool: VoxjValuePool = serde_json::from_str(text).unwrap();
        assert_eq!(serde_json::to_string(&value_pool).unwrap(), text);
    }

    #[test]
    fn int_values_beyond_the_cap_reject() {
        let max_magnitude: i64 = (1 << 53) - 1;
        for values in [
            json!([max_magnitude + 1]),
            json!([-max_magnitude - 1]),
            json!([u64::MAX]),
        ] {
            let wire = json!({ "kind": "int", "values": values });
            assert!(serde_json::from_value::<VoxjValuePool>(wire).is_err());
        }
        for values in [json!([max_magnitude]), json!([-max_magnitude])] {
            let wire = json!({ "kind": "int", "values": values });
            assert!(serde_json::from_value::<VoxjValuePool>(wire).is_ok());
        }
        let wire = json!({ "kind": "vec-2-int", "values": [[0, max_magnitude + 1]] });
        assert!(serde_json::from_value::<VoxjValuePool>(wire).is_err());
        assert!(serde_json::to_value(VoxjValuePool::Int(vec![max_magnitude + 1])).is_err());
    }

    #[test]
    fn non_integer_int_spellings_reject() {
        for values in ["[3.0]", "[3e0]", r#"["inf"]"#, r#"["-inf"]"#] {
            let text = format!(r#"{{"kind":"int","values":{values}}}"#);
            assert!(serde_json::from_str::<VoxjValuePool>(&text).is_err());
        }
    }

    #[test]
    fn stray_bound_keys_reject() {
        for key in ["min", "max"] {
            let mut wire = json!({ "kind": "float", "values": [0.5] });
            wire.as_object_mut()
                .unwrap()
                .insert(key.to_owned(), Value::String("none".to_owned()));
            assert!(serde_json::from_value::<VoxjValuePool>(wire).is_err());
        }
    }

    #[test]
    fn old_color_kinds_reject() {
        for (kind, values) in [
            ("srgb-hex", json!(["#FF0000"])),
            ("srgba-hex", json!(["#FF0000FF"])),
            ("srgb-float", json!([[1, 0, 0]])),
            ("srgba-float", json!([[1, 0, 0, 1]])),
            ("linear-rgb-float", json!([[1, 0, 0]])),
            ("linear-rgba-float", json!([[1, 0, 0, 1]])),
        ] {
            let wire = json!({ "kind": kind, "values": values });
            assert!(serde_json::from_value::<VoxjValuePool>(wire).is_err());
        }
    }

    #[test]
    fn vector_length_mismatches_reject() {
        for wire in [
            json!({ "kind": "vec-3-float", "values": [[0, 0]] }),
            json!({ "kind": "vec-3-float", "values": [[0, 0, 0, 0]] }),
            json!({ "kind": "vec-2-int", "values": [3] }),
        ] {
            assert!(serde_json::from_value::<VoxjValuePool>(wire).is_err());
        }
    }
}
