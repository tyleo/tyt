use crate::VMaxObjectState;
use vmax::VMaxContentsVmaxbFile;

/// Captures the editor state of a decoded `.vmaxb` (everything but its voxel
/// `snapshots`) for storage in the `voxel-max` ext.
pub fn object_state_from_data(data: &VMaxContentsVmaxbFile) -> VMaxObjectState {
    VMaxObjectState {
        uuid: data.uuid.clone(),
        v: data.v,
        tools: data.tools.clone(),
        brush: data.brush.clone(),
        cam: data.cam.clone(),
    }
}
