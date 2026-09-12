use crate::{SceneCameraSource, VMaxColorFormat};

/// Options for writing a Voxel Max document. The default stores palette
/// colors as PNG and keeps the ext's scene camera.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxWriteOptions {
    /// Where each palette's colors are stored.
    pub color_format: VMaxColorFormat,

    /// The scene camera the document opens with.
    pub scene_camera: SceneCameraSource,
}
