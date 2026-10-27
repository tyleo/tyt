/// The space a color reading measures distance in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    /// OKLab perceptual distance.
    Oklab,

    /// CIELAB distance.
    Lab,

    /// Naive distance on the sRGB-encoded components.
    Srgb,
}
