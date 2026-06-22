use crate::VoxjValue;
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};
#[cfg(feature = "serde")]
use std::fmt::{Formatter, Result as FmtResult};

/// An ordered set of key/value pairs: the object form of a [`VoxjValue`].
///
/// Insertion order is preserved on both serialization and deserialization so an
/// opaque `ext` namespace round-trips with its keys in their original order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjMap(pub Vec<(String, VoxjValue)>);

#[cfg(feature = "serde")]
impl Serialize for VoxjMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in &self.0 {
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
                let mut entries = Vec::with_capacity(access.size_hint().unwrap_or(0));
                while let Some(entry) = access.next_entry::<String, VoxjValue>()? {
                    entries.push(entry);
                }
                Ok(VoxjMap(entries))
            }
        }

        deserializer.deserialize_map(VoxjMapVisitor)
    }
}
