#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
};
use std::collections::BTreeMap;
#[cfg(feature = "serde")]
use std::fmt;

/// Schemaless plist value: dictionary, array, string, integer, real, boolean,
/// or data blob. Holds undocumented per-command undo/redo payloads (`ec.comt`,
/// the `ssnapshots`/`osnapshots` bodies, the `task` transactions) as untyped
/// `VMaxValue` (round-trips unchanged).
#[derive(Clone, Debug, PartialEq)]
pub enum VMaxValue {
    /// A keyed dictionary; keys are sorted for a deterministic round-trip.
    Dictionary(BTreeMap<String, VMaxValue>),

    /// An ordered array.
    Array(Vec<VMaxValue>),

    /// A string.
    String(String),

    /// An integer.
    Integer(i64),

    /// A real (floating-point) number.
    Real(f64),

    /// A boolean.
    Boolean(bool),

    /// A raw data blob.
    Data(Vec<u8>),
}

impl Default for VMaxValue {
    /// Empty dictionary; overwritten on decode.
    fn default() -> Self {
        VMaxValue::Dictionary(BTreeMap::new())
    }
}

#[cfg(feature = "serde")]
impl Serialize for VMaxValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            VMaxValue::Dictionary(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (key, value) in entries {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            VMaxValue::Array(items) => {
                let mut seq = serializer.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            VMaxValue::String(value) => serializer.serialize_str(value),
            VMaxValue::Integer(value) => serializer.serialize_i64(*value),
            VMaxValue::Real(value) => serializer.serialize_f64(*value),
            VMaxValue::Boolean(value) => serializer.serialize_bool(*value),
            VMaxValue::Data(value) => serializer.serialize_bytes(value),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VMaxValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Maps each plist primitive to its [`VMaxValue`] variant via
        /// `deserialize_any`.
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = VMaxValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a plist value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Integer(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Integer(i64::try_from(value).unwrap_or(i64::MAX)))
            }

            fn visit_f64<E>(self, value: f64) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Real(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<VMaxValue, E> {
                Ok(VMaxValue::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<VMaxValue, E> {
                Ok(VMaxValue::String(value))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Data(value.to_vec()))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<VMaxValue, E> {
                Ok(VMaxValue::Data(value))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<VMaxValue, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(VMaxValue::Array(items))
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<VMaxValue, A::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, VMaxValue>()? {
                    entries.insert(key, value);
                }
                Ok(VMaxValue::Dictionary(entries))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
