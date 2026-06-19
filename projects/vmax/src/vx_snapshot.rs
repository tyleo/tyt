use crate::VXStorage;
use serde::{Deserialize, Serialize};

/// A single entry in a `VXObjectData`'s `snapshots` array.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VXSnapshot {
    pub s: VXStorage,
}
