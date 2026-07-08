use clap::ValueEnum;

/// Distance metric for comparing palette colors.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ColorSpace {
    /// OKLab perceptual distance.
    #[value(name = "oklab")]
    Oklab,

    /// CIELAB distance.
    #[value(name = "lab")]
    Lab,

    /// sRGB distance.
    #[value(name = "srgb")]
    Srgb,
}
