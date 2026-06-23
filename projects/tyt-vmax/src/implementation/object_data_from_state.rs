use crate::VMaxObjectState;
use vmax::{VMaxSerdeContentsVmaxbFile, VMaxSerdeSnapshot};

/// Rebuilds a full `.vmaxb` payload from a preserved [`VMaxObjectState`] plus
/// regenerated voxel `snapshots`.
pub fn object_data_from_state(
    state: VMaxObjectState,
    snapshots: Vec<VMaxSerdeSnapshot>,
) -> VMaxSerdeContentsVmaxbFile {
    VMaxSerdeContentsVmaxbFile {
        snapshots,
        uuid: state.uuid,
        v: state.v,
        tools: state.tools,
        brush: state.brush,
        cam: state.cam,
    }
}
