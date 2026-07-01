/// Error diffusion applied when snapping voxel samples to the reduced palette.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dither {
    /// No diffusion; snap each sample to its nearest reduced value.
    #[default]
    None,

    /// Floyd-Steinberg diffusion in 3D voxel order.
    FloydSteinberg,

    /// Ordered, threshold-matrix dithering.
    Ordered,
}
