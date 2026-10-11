use crate::{SceneCameraSource, VMaxColorFormat, VMaxObjectSize};

/// Options for writing a Voxel Max document. The default stores palette
/// colors as PNG, keeps the ext's scene camera, and selects object sizes
/// automatically.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxWriteOptions {
    /// Where each palette's colors are stored.
    pub color_format: VMaxColorFormat,

    /// The scene camera the document opens with.
    pub scene_camera: SceneCameraSource,

    /// The editable cube each voxel object occupies.
    pub object_size: VMaxObjectSize,
}
