#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, de::IgnoredAny};

/// A value read and discarded: editor state, such as a contents file's brush
/// palette, that the model names so a strict read accepts it, keeps no part
/// of, and never writes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VMaxDiscarded;

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VMaxDiscarded {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        IgnoredAny::deserialize(deserializer).map(|_| VMaxDiscarded)
    }
}
