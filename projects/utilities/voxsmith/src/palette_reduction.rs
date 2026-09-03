use crate::{ColorSpace, Dither, ReductionMethod};

/// How [`reduce_palette`](crate::reduce_palette) reduces a palette.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaletteReduction {
    /// The most materials the palette may keep.
    pub max_materials: usize,

    /// The clustering algorithm.
    pub method: ReductionMethod,

    /// The color space colors are compared in.
    pub space: ColorSpace,

    /// Error diffusion applied when snapping samples to the reduced palette.
    pub dither: Dither,

    /// Keep value-pool values the reduction leaves unreferenced.
    pub keep_unused_values: bool,
}
