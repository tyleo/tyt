use crate::{FALLBACK_CONTENT_VERSION, VMaxExt, VMaxExtObjectState, place_object};
use std::collections::HashSet;
use vmax::VMaxCamera;
use voxcore::VoxObject;

/// The kept state of an object the document never carried, built against the
/// entries `ext` already holds so its contents UUID is fresh. It takes a camera
/// framed on the object's content center, which Voxel Max's object decoder
/// requires. The synthesizer and the retain hook both build entries here.
pub fn synthesized_object_state(ext: &VMaxExt, object: &VoxObject) -> VMaxExtObjectState {
    let uuids: HashSet<&str> = ext
        .object_states
        .values()
        .map(|entry| entry.uuid.as_str())
        .collect();
    let uuid = (0..)
        .map(synth_object_uuid)
        .find(|uuid| !uuids.contains(uuid.as_str()))
        .expect("a fresh index exists");
    let (_, placement) = place_object(&object.yup_to_zup());
    VMaxExtObjectState {
        uuid,
        v: FALLBACK_CONTENT_VERSION,
        cam: Some(default_camera(placement.center)),
        extent_order: None,
        camera_reference_center: Some(placement.center),
    }
}

/// A syntactically valid, deterministic UUID for a synthesized object's
/// contents file, in a lane of its own so it never collides with a synthesized
/// node's UUID.
fn synth_object_uuid(index: usize) -> String {
    format!("00000000-0000-0001-0000-{:012X}", index + 1)
}

/// Default editor `cam` for a synthesized object. A single-object document
/// opens straight into this object-editor view, so the camera must orbit the
/// object's content, not the corner of its 0..256 internal grid: `target` is
/// the content center in internal-grid coordinates, the object's `e_c`,
/// matching the rig Voxel Max writes when it frames the object. The rest is a
/// neutral framed-view rig.
fn default_camera(target: [f64; 3]) -> VMaxCamera {
    VMaxCamera {
        wa: 0.0,
        ha: 0.1959133446216583,
        da: 0.0,
        lwa: 0.25,
        lha: 1.820913314819336,
        lda: 0.0,
        px: 0.0,
        py: 0.0,
        z: 512.0,
        o: target,
        ..Default::default()
    }
}
