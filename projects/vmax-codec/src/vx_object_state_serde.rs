use crate::{VXBrushSerde, VXCameraSerde, VXObjectDataSerde, VXSnapshotSerde, VXToolsSerde};
use serde::{Deserialize, Serialize};

/// The per-object Voxel Max editor state preserved in the `voxel-max` ext so
/// `from-voxj` can rebuild the `.vmaxb` top level Voxel Max requires to import an
/// object: the content `uuid`/version plus the `tools`/`brush`/`cam` state. The
/// geometry lives in the voxj document, so the voxel `snapshots` are regenerated
/// on reconstruction rather than stored here.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXObjectStateSerde {
    /// Object content UUID (`uuid`).
    pub uuid: String,
    /// Codable version (`v`).
    pub v: i64,
    /// Tool state (`tools`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<VXToolsSerde>,
    /// Brush palette (`brush`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brush: Option<VXBrushSerde>,
    /// Per-object camera (`cam`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cam: Option<VXCameraSerde>,
}

impl VXObjectStateSerde {
    /// Captures the editor state of a decoded `.vmaxb` — everything but its voxel
    /// `snapshots` — for storage in the `voxel-max` ext.
    pub fn from_object_data(data: &VXObjectDataSerde) -> Self {
        Self {
            uuid: data.uuid.clone(),
            v: data.v,
            tools: data.tools.clone(),
            brush: data.brush.clone(),
            cam: data.cam.clone(),
        }
    }

    /// Rebuilds a full `.vmaxb` payload from this state plus regenerated voxel
    /// `snapshots`.
    pub fn into_object_data(self, snapshots: Vec<VXSnapshotSerde>) -> VXObjectDataSerde {
        VXObjectDataSerde {
            snapshots,
            uuid: self.uuid,
            v: self.v,
            tools: self.tools,
            brush: self.brush,
            cam: self.cam,
        }
    }
}
