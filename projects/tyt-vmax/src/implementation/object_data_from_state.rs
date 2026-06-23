use crate::VoxelMaxObjectState;
use vmax::{VMaxSerdeContentsVmaxbFile, VMaxSerdeSnapshot};

/// Rebuilds a full `.vmaxb` payload from a preserved [`VoxelMaxObjectState`] plus
/// regenerated voxel `snapshots`.
pub fn object_data_from_state(
    state: VoxelMaxObjectState,
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
