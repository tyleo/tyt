use clap::ValueEnum;

/// Error-diffusion method when snapping voxel samples to new palette values.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Dither {
    /// No diffusion; snap each sample to the nearest value.
    #[default]
    #[value(name = "none")]
    None,
    /// Floyd-Steinberg diffusion in 3D voxel order.
    #[value(name = "floyd-steinberg")]
    FloydSteinberg,
    /// Ordered, threshold-matrix dithering.
    #[value(name = "ordered")]
    Ordered,
}
