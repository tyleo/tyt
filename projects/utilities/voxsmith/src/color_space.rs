/// The color space a palette reduction compares colors in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSpace {
    /// OKLab perceptual distance.
    Oklab,

    /// CIELAB distance.
    Lab,

    /// Naive distance on the stored sRGB components.
    Srgb,
}
