use vmax::{VXObjectData, VXObjectState, VXSnapshot};

/// Rebuilds a full `.vmaxb` payload from a preserved [`VXObjectState`] plus
/// regenerated voxel `snapshots`.
pub fn object_data_from_state(state: VXObjectState, snapshots: Vec<VXSnapshot>) -> VXObjectData {
    VXObjectData {
        snapshots,
        uuid: state.uuid,
        v: state.v,
        tools: state.tools,
        brush: state.brush,
        cam: state.cam,
    }
}
