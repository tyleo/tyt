/// How often a light's shadow ray is cast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderShadow {
    /// No shadow.
    None,

    /// One ray per pixel, a crisp edge across faces.
    PerPixel,

    /// One ray per face from its center, a crisp staircase at voxel
    /// resolution.
    PerFace,

    /// One ray per face corner, blended across the face, a soft staircase.
    PerCorner,
}
