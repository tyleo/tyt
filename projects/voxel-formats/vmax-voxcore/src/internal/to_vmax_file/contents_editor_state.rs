use crate::{ObjectPlacement, VMaxExtObjectState};
use vmax::VMaxContentsVmaxbFile;

/// The editor state a contents file carries, without its snapshots: the
/// entry's session as it is, with the canvas `vp` re-scoped to the derived
/// build volume.
pub(crate) fn contents_editor_state(
    object_state: &VMaxExtObjectState,
    placement: &ObjectPlacement,
) -> VMaxContentsVmaxbFile {
    let mut tools = object_state.tools.clone();
    if let Some(tools) = tools.as_mut() {
        tools.vp = Some(placement.view_box.clone());
    }
    VMaxContentsVmaxbFile {
        snapshots: Vec::new(),
        uuid: object_state.uuid.clone(),
        v: object_state.v,
        tools,
        brush: object_state.brush.clone(),
        cam: object_state.cam.clone(),
        pal: None,
    }
}
