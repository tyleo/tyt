use crate::commands::{ColorSpace, Dither, QuantizeMethod};

/// A resolved palette-reduction configuration: the material cap and the shared
/// method, space, and dither controls. Passed to a reduction operation.
#[derive(Clone, Copy, Debug)]
pub struct PaletteReduction {
    /// Cap on the palette's material count, or `None` for no cap.
    pub max_materials: Option<usize>,

    /// Clustering algorithm.
    pub method: QuantizeMethod,

    /// Color space colors are compared in.
    pub space: ColorSpace,

    /// Error diffusion applied when snapping samples to the reduced palette.
    pub dither: Dither,

    /// Keep pool values the reduction leaves unreferenced.
    pub keep_unused_values: bool,
}
