use crate::GoxlDict;

/// A `CAMR` camera.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GoxlCamera {
    /// Camera name.
    pub name: String,

    /// Rotation distance from the target.
    pub distance: f32,

    /// Whether the projection is orthographic.
    pub orthographic: bool,

    /// `4 x 4` camera-to-world transform, as 16 floats in stored order.
    pub transform: [[f32; 4]; 4],

    /// Whether this is the file's active camera (the `active` flag key).
    pub active: bool,

    /// Dict keys this crate does not model, preserved verbatim.
    pub extra: GoxlDict,
}
