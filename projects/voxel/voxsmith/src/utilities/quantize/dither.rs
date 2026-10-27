/// Error diffusion applied when snapping voxel samples to representatives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dither {
    /// No diffusion; snap each sample to its nearest representative.
    None,

    /// Floyd-Steinberg diffusion in 3D voxel order.
    FloydSteinberg,

    /// Ordered, threshold-matrix dithering.
    Ordered,
}
