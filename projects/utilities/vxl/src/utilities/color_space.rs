use clap::ValueEnum;

/// Distance metric for comparing palette colors.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ColorSpace {
    /// OKLab perceptual distance.
    #[default]
    #[value(name = "oklab")]
    Oklab,
    /// CIELAB distance.
    #[value(name = "lab")]
    Lab,
    /// RGB distance.
    #[value(name = "rgb")]
    Rgb,
}
