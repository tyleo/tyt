use crate::commands::CameraView;
use voxconv::vmax::{SceneCameraSource, VMaxSceneCamera};

/// The top-corner scene camera `--camera corner` writes: the empty camera's
/// framing at the origin, rotated to a three-quarter view looking down at the
/// model from a corner. Specific to this CLI.
const TOP_CORNER_CAMERA: VMaxSceneCamera = VMaxSceneCamera {
    da: 0.0,
    ha: 0.19591325521469116,
    lda: 0.0,
    lha: 1.820913314819336,
    lwa: 0.5,
    o: [0.0, 0.0, 0.0],
    px: 0.0,
    py: 0.0,
    wa: 0.25,
    z: 512.0,
};

/// The scene camera the writer takes for a `--camera` choice. Omitted, the
/// writer keeps the ext's camera.
pub(crate) fn resolve_scene_camera(camera: Option<CameraView>) -> SceneCameraSource {
    match camera {
        None | Some(CameraView::Ext) => SceneCameraSource::Ext,
        Some(CameraView::Empty) => SceneCameraSource::Empty,
        Some(CameraView::Corner) => SceneCameraSource::Camera(TOP_CORNER_CAMERA),
    }
}
