use crate::VoxjMapEntry;
#[cfg(feature = "serde")]
use crate::VoxjValue;
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error as DeError, MapAccess, Visitor},
    ser::{Error as SerError, SerializeMap},
};
#[cfg(feature = "serde")]
use std::fmt::{Formatter, Result as FmtResult};

/// An ordered set of key/value pairs: the object form of a [`VoxjValue`].
///
/// Insertion order is preserved through serialization, so the `ext` namespace
/// round-trips with its key order intact. Keys are unique: reading or writing
/// a repeated key is an error, never a last-wins resolution.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjMap(Vec<VoxjMapEntry>);

impl VoxjMap {
    /// A map of `entries`, in their order.
    pub fn new(entries: Vec<VoxjMapEntry>) -> Self {
        Self(entries)
    }

    /// The entries, in insertion order.
    pub fn entries(&self) -> &[VoxjMapEntry] {
        &self.0
    }

    /// The entries, taken out of the map.
    pub fn into_entries(self) -> Vec<VoxjMapEntry> {
        self.0
    }
}

#[cfg(feature = "serde")]
impl Serialize for VoxjMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (index, VoxjMapEntry { key, value }) in self.0.iter().enumerate() {
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
impl<'de> Deserialize<'de> for VoxjMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VoxjMapVisitor;

        impl<'de> Visitor<'de> for VoxjMapVisitor {
            type Value = VoxjMap;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("a Voxel Json object")
            }

            fn visit_map<A>(self, mut access: A) -> Result<VoxjMap, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries: Vec<VoxjMapEntry> =
                    Vec::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((key, value)) = access.next_entry::<String, VoxjValue>()? {
                    if entries.iter().any(|existing| existing.key == key) {
                        return Err(DeError::custom(format!(
                            "json object key `{key}` must be unique"
                        )));
                    }
                    entries.push(VoxjMapEntry { key, value });
                }
                Ok(VoxjMap(entries))
            }
        }

        deserializer.deserialize_map(VoxjMapVisitor)
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{VoxjMap, VoxjMapEntry, VoxjValue};

    /// A map holding the same key twice, buildable only in memory.
    fn repeated() -> VoxjMap {
        VoxjMap::new(vec![
            VoxjMapEntry {
                key: "k".to_owned(),
                value: VoxjValue::Number(1.0),
            },
            VoxjMapEntry {
                key: "k".to_owned(),
                value: VoxjValue::Number(2.0),
            },
        ])
    }

    #[test]
    fn a_repeated_key_errors_on_read() {
        // Last-wins would silently drop the first value.
        assert!(serde_json::from_str::<VoxjMap>(r#"{"k": 1, "k": 2}"#).is_err());
    }

    #[test]
    fn a_repeated_key_errors_on_write() {
        assert!(serde_json::to_string(&repeated()).is_err());
    }

    #[test]
    fn unique_keys_round_trip_in_order() {
        let map = VoxjMap::new(vec![
            VoxjMapEntry {
                key: "b".to_owned(),
                value: VoxjValue::Number(1.0),
            },
            VoxjMapEntry {
                key: "a".to_owned(),
                value: VoxjValue::Number(2.0),
            },
        ]);
        let text = serde_json::to_string(&map).unwrap();
        assert_eq!(text, r#"{"b":1,"a":2}"#);
        assert_eq!(serde_json::from_str::<VoxjMap>(&text).unwrap(), map);
    }
}
