/// Error diffusion applied when snapping voxel samples to the reduced palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dither {
    /// No diffusion; snap each sample to its nearest reduced value.
    None,

    /// Floyd-Steinberg diffusion in 3D voxel order.
    FloydSteinberg,

    /// Ordered, threshold-matrix dithering.
    Ordered,
}
