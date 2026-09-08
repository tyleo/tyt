use crate::{SceneCameraSource, VMaxColorFormat};

/// Options for writing a Voxel Max document. The default stores palette
/// colors as PNG and keeps the scene camera the path produces.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxWriteOptions {
    /// Where each palette's colors are stored.
    pub color_format: VMaxColorFormat,

    /// Overrides the scene camera the document opens with. `None` keeps the
    /// camera the path produces: the ext's on the typed path, the empty
    /// default in synthesis.
    pub scene_camera: Option<SceneCameraSource>,
}
