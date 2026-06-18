use crate::VXStorageSerde;
use serde::{Deserialize, Serialize};

/// A single entry in a `VXObjectData`'s `snapshots` array.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VXSnapshotSerde {
    pub s: VXStorageSerde,
}
