/// The color space a palette reduction compares colors in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpace {
    /// OKLab perceptual distance.
    #[default]
    Oklab,

    /// CIELAB distance.
    Lab,

    /// Naive distance on the stored sRGB components.
    Rgb,
}
