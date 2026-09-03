use crate::{NoneOr, PositiveCount, cli_value_parser, commands::PaletteReductionOptions};
use clap::{ArgAction, Args};
use voxsmith::{ColorSpace, Dither, PaletteReduction, ReductionMethod};

/// `voxelize`'s color-quantization controls: the shared method, space, and
/// dither under a `--quantize-*` prefix, plus its own material cap. Maps onto
/// [`PaletteReductionOptions`] to resolve.
#[derive(Clone, Debug, Args)]
pub struct QuantizeOptions {
    /// Clustering algorithm for the reduction.
    #[arg(
        value_name = "quantize-method",
        long = "quantize-method",
        default_value = "median-cut",
        value_parser = cli_value_parser::<ReductionMethod>()
    )]
    method: ReductionMethod,

    /// Color space the reduction compares colors in.
    #[arg(
        value_name = "quantize-space",
        long = "quantize-space",
        default_value = "oklab",
        value_parser = cli_value_parser::<ColorSpace>()
    )]
    space: ColorSpace,

    /// Error diffusion applied when snapping samples to the reduced palette.
    #[arg(
        value_name = "quantize-dither",
        long = "quantize-dither",
        default_value = "none",
        value_parser = cli_value_parser::<Dither>()
    )]
    dither: Dither,

    /// Keep palette values the reduction leaves unreferenced. Off by default, so
    /// a quantized palette keeps only the colors its materials use, fitting a
    /// small budget such as Voxel Max's 255 colors; pass it to keep every sampled
    /// value.
    #[arg(
        value_name = "quantize-keep-unused-values",
        long = "quantize-keep-unused-values",
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    keep_unused_values: bool,

    /// Most materials the palette may hold; sampling over this reduces to it.
    /// `none` disables the cap. Lives here, not in the shared controls, so
    /// voxelize can default it to 256.
    #[arg(
        value_name = "quantize-max-materials",
        long = "quantize-max-materials",
        default_value = "256"
    )]
    max_materials: NoneOr<PositiveCount>,
}

impl QuantizeOptions {
    /// Maps the shared controls onto [`PaletteReductionOptions`] and resolves
    /// them with the material cap, `None` under `--quantize-max-materials none`.
    pub fn resolve(&self) -> Option<PaletteReduction> {
        PaletteReductionOptions {
            method: self.method,
            space: self.space,
            dither: self.dither,
            keep_unused_values: self.keep_unused_values,
        }
        .resolve(self.max_materials.value().map(|count| count.0))
    }
}
