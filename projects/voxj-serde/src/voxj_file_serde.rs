use crate::VoxjMainSerde;
use serde::{Deserialize, Serialize};
use voxj::VoxjFile;

/// Serde-compatible parity type for [`VoxjFile`]: the root of a Voxel Json
/// document on the wire.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VoxjFileSerde {
    pub version: u32,
    pub main: VoxjMainSerde,
}

impl From<VoxjFile> for VoxjFileSerde {
    fn from(v: VoxjFile) -> Self {
        Self {
            version: v.version,
            main: v.main.into(),
        }
    }
}

impl From<VoxjFileSerde> for VoxjFile {
    fn from(v: VoxjFileSerde) -> Self {
        Self {
            version: v.version,
            main: v.main.into(),
        }
    }
}
