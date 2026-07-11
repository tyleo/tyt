use crate::{ColorSpace, Dither, PaletteReduction, QuantizeMethod};
use clap::{ArgAction, Args};

/// The shared palette-reduction controls: clustering method, color space,
/// dither, and unused-value policy. Voxelize maps its own `--quantize-*` flags
/// onto these and pairs in a material cap to resolve a [`PaletteReduction`].
#[derive(Clone, Debug, Args)]
pub struct PaletteReductionOptions {
    /// Clustering algorithm for the reduction.
    #[arg(value_name = "method", long, default_value = "median-cut")]
    pub(crate) method: QuantizeMethod,

    /// Color space the reduction compares colors in.
    #[arg(value_name = "space", long, default_value = "oklab")]
    pub(crate) space: ColorSpace,

    /// Error diffusion applied when snapping samples to the reduced palette.
    #[arg(value_name = "dither", long, default_value = "none")]
    pub(crate) dither: Dither,

    /// Keep palette values the reduction leaves unreferenced. Off by default, so
    /// a quantized palette keeps only the colors its materials use, fitting a
    /// small budget such as Voxel Max's 255 colors; pass it to keep every sampled
    /// value.
    #[arg(
        value_name = "keep-unused-values",
        long,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    pub(crate) keep_unused_values: bool,
}

impl PaletteReductionOptions {
    /// Pairs these controls with a material cap into a [`PaletteReduction`].
    pub fn resolve(&self, max_materials: Option<usize>) -> PaletteReduction {
        PaletteReduction {
            max_materials,
            method: self.method,
            space: self.space,
            dither: self.dither,
            keep_unused_values: self.keep_unused_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PaletteReductionOptions;
    use clap::Parser;

    /// A throwaway command flattening the reduction options, so their flags
    /// parse as they do on `voxelize`.
    #[derive(Parser)]
    struct Harness {
        #[command(flatten)]
        options: PaletteReductionOptions,
    }

    /// Whether the reduction resolved from `args` keeps unused values.
    fn keep_unused_values(args: &[&str]) -> bool {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        Harness::try_parse_from(argv)
            .unwrap()
            .options
            .resolve(Some(256))
            .keep_unused_values
    }

    #[test]
    fn drops_unused_by_default() {
        assert!(!keep_unused_values(&[]));
    }

    #[test]
    fn keep_unused_values_opts_out() {
        assert!(keep_unused_values(&["--keep-unused-values"]));
        assert!(keep_unused_values(&["--keep-unused-values", "true"]));
    }

    #[test]
    fn keep_unused_values_false_still_drops() {
        assert!(!keep_unused_values(&["--keep-unused-values", "false"]));
    }
}
