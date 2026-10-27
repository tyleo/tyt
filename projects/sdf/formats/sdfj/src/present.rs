use serde::{Deserialize, Deserializer};

/// Deserializes a present optional field. An explicit `null` fails to read.
pub fn present<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}
