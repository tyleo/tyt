use crate::VXObjectState;
use vmax::VXObjectData;

/// Captures the editor state of a decoded `.vmaxb` (everything but its voxel
/// `snapshots`) for storage in the `voxel-max` ext.
pub fn object_state_from_data(data: &VXObjectData) -> VXObjectState {
    VXObjectState {
        uuid: data.uuid.clone(),
        v: data.v,
        tools: data.tools.clone(),
        brush: data.brush.clone(),
        cam: data.cam.clone(),
    }
}
