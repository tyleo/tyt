use crate::{FALLBACK_CONTENT_VERSION, VMaxExt, VMaxExtObjectState};
use std::collections::HashSet;

/// The kept state of an object the document never carried, built against the
/// entries `ext` already holds so its contents UUID is fresh. It keeps no
/// camera, so the write frames the object's content. The synthesizer and the
/// retain hook both build entries here.
pub fn synthesized_object_state(ext: &VMaxExt) -> VMaxExtObjectState {
    let uuids: HashSet<&str> = ext
        .object_states
        .values()
        .map(|entry| entry.uuid.as_str())
        .collect();
    let uuid = (0..)
        .map(synth_object_uuid)
        .find(|uuid| !uuids.contains(uuid.as_str()))
        .expect("a fresh index exists");
    VMaxExtObjectState {
        uuid,
        v: FALLBACK_CONTENT_VERSION,
        cam: None,
        extent_order: None,
        camera_reference_center: None,
    }
}

/// A syntactically valid, deterministic UUID for a synthesized object's
/// contents file, in a lane of its own so it never collides with a synthesized
/// node's UUID.
fn synth_object_uuid(index: usize) -> String {
    format!("00000000-0000-0001-0000-{:012X}", index + 1)
}
