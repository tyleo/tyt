use crate::{ColorSpace, Dither, MaxPaletteMaterials, PaletteReduction, QuantizeMethod};
use clap::Args;

/// The shared palette-reduction controls: clustering method, color space, and
/// dither. Flattened by the commands that reduce a palette (`voxelize` and the
/// palette-reduction commands), each pairing it with its own cap flag.
#[derive(Clone, Debug, Args)]
pub struct PaletteReductionOptions {
    /// Clustering algorithm for the reduction.
    #[arg(value_name = "method", long, default_value = "median-cut")]
    method: QuantizeMethod,

    /// Color space the reduction compares colors in.
    #[arg(value_name = "space", long, default_value = "oklab")]
    space: ColorSpace,

    /// Error diffusion applied when snapping samples to the reduced palette.
    #[arg(value_name = "dither", long, default_value = "none")]
    dither: Dither,
}

impl PaletteReductionOptions {
    /// Pairs these controls with a material cap into a [`PaletteReduction`].
    pub fn resolve(&self, max_materials: MaxPaletteMaterials) -> PaletteReduction {
        PaletteReduction {
            max_materials: max_materials.limit(),
            method: self.method,
            space: self.space,
            dither: self.dither,
        }
    }
}
