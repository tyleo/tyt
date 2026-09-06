use vmax_voxcore::{SceneCameraSource, VMaxColorFormat};

/// Writer options for a Voxel Max `.vmax` package.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxWriteOptions {
    /// Where each palette's colors are stored.
    pub color_format: VMaxColorFormat,

    /// The scene camera the document opens with. `None` keeps the ext's
    /// camera when the state carries one, else the empty default.
    pub scene_camera: Option<SceneCameraSource>,
}

/// PNG palette colors and no camera override.
impl Default for VMaxWriteOptions {
    fn default() -> Self {
        Self {
            color_format: VMaxColorFormat::Png,
            scene_camera: None,
        }
    }
}
