use crate::{SdfjMapEntry, SdfjValue};
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, MapAccess, Visitor},
    ser::{Error as SerError, SerializeMap},
};
#[cfg(feature = "serde")]
use std::{
    fmt::{Formatter, Result as FmtResult},
    marker::PhantomData,
};

/// An ordered set of key/value pairs: a JSON object.
///
/// Serialization keeps the insertion order. Reading or writing a repeated key
/// errors instead of keeping the last value.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfjMap<TValue = SdfjValue>(Vec<SdfjMapEntry<TValue>>);

impl<TValue> SdfjMap<TValue> {
    /// A map of `entries`, in their order.
    pub fn new(entries: Vec<SdfjMapEntry<TValue>>) -> Self {
        Self(entries)
    }

    /// The entries, in insertion order.
    pub fn entries(&self) -> &[SdfjMapEntry<TValue>] {
        &self.0
    }

    /// The entries, taken out of the map.
    pub fn into_entries(self) -> Vec<SdfjMapEntry<TValue>> {
        self.0
    }

    /// Whether the map holds no entry.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<TValue> Default for SdfjMap<TValue> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

#[cfg(feature = "serde")]
impl<TValue: Serialize> Serialize for SdfjMap<TValue> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        for (index, SdfjMapEntry { key, value }) in self.0.iter().enumerate() {
            if self.0[..index].iter().any(|existing| existing.key == *key) {
                return Err(SerError::custom(format!(
                    "json object key `{key}` must be unique"
                )));
            }

            map.serialize_entry(key, value)?;
        }

        map.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, TValue: Deserialize<'de>> Deserialize<'de> for SdfjMap<TValue> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SdfjMapVisitor<TValue>(PhantomData<TValue>);

        impl<'de, TValue: Deserialize<'de>> Visitor<'de> for SdfjMapVisitor<TValue> {
            type Value = SdfjMap<TValue>;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A>(self, mut access: A) -> Result<SdfjMap<TValue>, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries: Vec<SdfjMapEntry<TValue>> =
                    Vec::with_capacity(access.size_hint().unwrap_or(0));

                while let Some((key, value)) = access.next_entry::<String, TValue>()? {
                    if entries.iter().any(|existing| existing.key == key) {
                        return Err(DeError::custom(format!(
                            "json object key `{key}` must be unique"
                        )));
                    }

                    entries.push(SdfjMapEntry { key, value });
                }

                Ok(SdfjMap(entries))
            }
        }

        deserializer.deserialize_map(SdfjMapVisitor(PhantomData))
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{SdfjMap, SdfjMapEntry, SdfjValue};

    #[test]
    fn a_repeated_key_errors_on_read() {
        assert!(serde_json::from_str::<SdfjMap>(r#"{"k": 1, "k": 2}"#).is_err());
    }

    #[test]
    fn a_repeated_key_errors_on_write() {
        let entry = SdfjMapEntry {
            key: "k".to_owned(),
            value: SdfjValue::Null,
        };

        let map = SdfjMap::new(vec![entry.clone(), entry]);

        assert!(serde_json::to_string(&map).is_err());
    }

    #[test]
    fn unique_keys_round_trip_in_order() {
        let text = r#"{"b":1,"a":2}"#;

        let map: SdfjMap = serde_json::from_str(text).unwrap();

        assert_eq!(serde_json::to_string(&map).unwrap(), text);
    }
}
