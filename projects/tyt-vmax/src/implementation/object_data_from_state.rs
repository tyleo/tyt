use crate::VMaxObjectState;
use vmax::{VMaxContentsVmaxbFile, VMaxSnapshot};

/// Rebuilds a full `.vmaxb` payload from a preserved [`VMaxObjectState`] plus
/// regenerated voxel `snapshots`.
pub fn object_data_from_state(
    state: VMaxObjectState,
    snapshots: Vec<VMaxSnapshot>,
) -> VMaxContentsVmaxbFile {
    VMaxContentsVmaxbFile {
        snapshots,
        uuid: state.uuid,
        v: state.v,
        tools: state.tools,
        brush: state.brush,
        cam: state.cam,
    }
}
