use crate::VXStorage;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single entry in a `VXObjectData`'s `snapshots` array.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VXSnapshot {
    pub s: VXStorage,
}
