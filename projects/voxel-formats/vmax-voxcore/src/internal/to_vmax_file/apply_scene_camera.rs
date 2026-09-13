use crate::{SYNTH_CAMERA, SceneCameraSource};
use vmax::VMaxSceneJsonFile;

/// Applies the scene camera choice to the rebuilt scene. `Ext` leaves the
/// camera the ext supplied.
pub(crate) fn apply_scene_camera(scene: &mut VMaxSceneJsonFile, scene_camera: SceneCameraSource) {
    match scene_camera {
        SceneCameraSource::Ext => {}
        SceneCameraSource::Empty => scene.cam = Some(SYNTH_CAMERA),
        SceneCameraSource::Camera(camera) => scene.cam = Some(camera),
    }
}
