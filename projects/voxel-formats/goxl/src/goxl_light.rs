use crate::GoxlDict;

/// The `LIGH` chunk: the scene light and shading settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoxlLight {
    /// Light pitch, in radians.
    pub pitch: f32,

    /// Light yaw, in radians.
    pub yaw: f32,

    /// Light intensity.
    pub intensity: f32,

    /// Whether the light direction is fixed relative to the camera.
    pub fixed: bool,

    /// Ambient light amount.
    pub ambient: f32,

    /// Shadow amount.
    pub shadow: f32,

    /// Dict keys this crate does not model, preserved verbatim.
    pub extra: GoxlDict,
}
