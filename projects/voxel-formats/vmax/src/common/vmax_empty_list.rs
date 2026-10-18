#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer,
    de::{Error as DeError, IgnoredAny, SeqAccess, Visitor},
};
#[cfg(feature = "serde")]
use std::fmt::{Formatter, Result as FmtResult};

/// A list Voxel Max writes empty and whose contents the model does not know,
/// such as a contents file's `bbr`. It reads only when empty and is never
/// written.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VMaxEmptyList;

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VMaxEmptyList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(EmptyListVisitor)
    }
}

/// Accepts a list with no elements.
#[cfg(feature = "serde")]
struct EmptyListVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for EmptyListVisitor {
    type Value = VMaxEmptyList;

    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
        formatter.write_str("an empty list")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        match seq.next_element::<IgnoredAny>()? {
            Some(_) => Err(DeError::custom(
                "a list the model does not know the contents of is not empty",
            )),

            None => Ok(VMaxEmptyList),
        }
    }
}
