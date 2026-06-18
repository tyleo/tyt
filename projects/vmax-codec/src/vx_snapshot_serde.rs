use crate::VXStorageSerde;
use serde::Deserialize;

/// A single entry in a `VXObjectData`'s `snapshots` array.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct VXSnapshotSerde {
    pub s: VXStorageSerde,
}
