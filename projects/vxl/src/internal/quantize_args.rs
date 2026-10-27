use crate::{Error, QuantizeProfile, Result, cli_value_parser};
use clap::Args;
use std::num::NonZeroUsize;
use voxcore::material::BASE_COLOR;
use voxsmith::utilities::{
    AlphaMode, ColorSpace, Dither, PartitionProperties, PropertyInterpretation, QuantizeOptions,
    ReductionMethod,
};

/// The flags `palette quantize` and `object voxels quantize` share, resolving
/// to voxsmith's [`QuantizeOptions`] over a profile.
#[derive(Clone, Debug, Args)]
pub struct QuantizeArgs {
    /// The most materials each quantized layer may sample afterward. Required
    /// unless a profile sets it.
    #[arg(value_name = "max-materials", long)]
    max_materials: Option<NonZeroUsize>,

    /// The property to cluster on. Defaults to `baseColor`.
    #[arg(value_name = "property", long)]
    property: Option<String>,

    /// How the property's values read as points. Defaults to `auto`.
    #[arg(
        value_name = "interpret-property",
        long,
        value_parser = cli_value_parser::<PropertyInterpretation>()
    )]
    interpret_property: Option<PropertyInterpretation>,

    /// How a four-component color's alpha takes part. Any other reading errors
    /// on it. Defaults to `partition`.
    #[arg(value_name = "alpha", long, value_parser = cli_value_parser::<AlphaMode>())]
    alpha: Option<AlphaMode>,

    /// Merges materials only when they agree on this property. `*` stands for
    /// every property other than `--property`. Replaces the profile's
    /// partitions. Repeatable.
    #[arg(value_name = "partition", long)]
    partition: Vec<String>,

    /// Clustering algorithm. Defaults to `median-cut`.
    #[arg(
        value_name = "method",
        long,
        value_parser = cli_value_parser::<ReductionMethod>()
    )]
    method: Option<ReductionMethod>,

    /// The space a color reading measures distance in. A numeric reading errors
    /// on it. Defaults to `oklab`.
    #[arg(value_name = "space", long, value_parser = cli_value_parser::<ColorSpace>())]
    space: Option<ColorSpace>,

    /// Error diffusion applied when snapping samples to representatives.
    /// Defaults to `none`.
    #[arg(value_name = "dither", long, value_parser = cli_value_parser::<Dither>())]
    dither: Option<Dither>,
}

impl QuantizeArgs {
    /// The [`QuantizeOptions`] these flags set over `profile`, each flag
    /// overriding the element it mirrors. Errors when neither sets
    /// `--max-materials` or when `--partition '*'` comes with a named
    /// partition.
    pub fn resolve(&self, profile: &QuantizeProfile) -> Result<QuantizeOptions> {
        let max_materials = self
            .max_materials
            .or(profile.max_materials)
            .ok_or_else(|| {
                Error::usage("`--max-materials` is required unless a profile sets `maxMaterials`")
            })?;

        let partition = match self.partition.is_empty() {
            true => &profile.partition,
            false => &self.partition,
        };

        Ok(QuantizeOptions {
            max_materials,
            property: self
                .property
                .clone()
                .or_else(|| profile.property.clone())
                .unwrap_or_else(|| BASE_COLOR.to_owned()),
            interpret_property: self
                .interpret_property
                .or(profile.interpret_property.map(|named| named.0))
                .unwrap_or(PropertyInterpretation::Auto),
            alpha: self.alpha.or(profile.alpha.map(|named| named.0)),
            partition: partition_properties(partition)?,
            method: self
                .method
                .or(profile.method.map(|named| named.0))
                .unwrap_or(ReductionMethod::MedianCut),
            space: self.space.or(profile.space.map(|named| named.0)),
            dither: self
                .dither
                .or(profile.dither.map(|named| named.0))
                .unwrap_or(Dither::None),
        })
    }
}

/// The partition properties `names` lists, where `*` stands for all.
fn partition_properties(names: &[String]) -> Result<PartitionProperties> {
    let named: Vec<_> = names.iter().filter(|name| *name != "*").cloned().collect();

    match named.len() < names.len() {
        true if !named.is_empty() => Err(Error::usage(
            "`--partition '*'` covers every other property, so it takes no named partition \
             beside it",
        )),

        true => Ok(PartitionProperties::All),

        false => Ok(PartitionProperties::Named(named)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{QuantizeArgs, QuantizeProfile, Result};
    use clap::Parser;
    use voxsmith::utilities::{Dither, PartitionProperties, QuantizeOptions, ReductionMethod};

    /// A command carrying only the quantize flags.
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        quantize: QuantizeArgs,
    }

    /// The options `args` resolve to over `profile`.
    fn resolve_over(args: &[&str], profile: &QuantizeProfile) -> Result<QuantizeOptions> {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().quantize.resolve(profile)
    }

    /// The options `args` resolve to after `--max-materials 4`, with no
    /// profile.
    fn resolve(args: &[&str]) -> Result<QuantizeOptions> {
        let mut argv = vec!["--max-materials", "4"];
        argv.extend_from_slice(args);
        resolve_over(&argv, &QuantizeProfile::default())
    }

    /// The profile `json` defines.
    fn profile(json: &str) -> QuantizeProfile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn max_materials_is_positive_and_required_without_a_profile() {
        assert!(Cli::try_parse_from(["cli", "--max-materials", "0"]).is_err());
        let error = resolve_over(&[], &QuantizeProfile::default())
            .unwrap_err()
            .to_string();
        assert!(error.contains("`--max-materials` is required"), "{error}");
    }

    #[test]
    fn defaults_leave_alpha_and_space_to_the_reading() {
        let options = resolve(&[]).unwrap();
        assert_eq!(options.max_materials.get(), 4);
        assert_eq!(options.property, "baseColor");
        assert_eq!(options.alpha, None);
        assert_eq!(options.space, None);
        assert_eq!(options.method, ReductionMethod::MedianCut);
        assert_eq!(options.partition, PartitionProperties::Named(Vec::new()));
    }

    #[test]
    fn partitions_repeat_and_star_stands_for_all() {
        assert_eq!(
            resolve(&["--partition", "metallic", "--partition", "ior"])
                .unwrap()
                .partition,
            PartitionProperties::Named(vec!["metallic".to_owned(), "ior".to_owned()])
        );
        assert_eq!(
            resolve(&["--partition", "*"]).unwrap().partition,
            PartitionProperties::All
        );
        assert!(resolve(&["--partition", "*", "--partition", "metallic"]).is_err());
    }

    #[test]
    fn a_profile_fills_what_the_flags_leave_and_the_flags_win() {
        let retro =
            profile(r#"{ "maxMaterials": 16, "dither": "ordered", "partition": ["metallic"] }"#);

        let options = resolve_over(&[], &retro).unwrap();
        assert_eq!(options.max_materials.get(), 16);
        assert_eq!(options.dither, Dither::Ordered);
        assert_eq!(
            options.partition,
            PartitionProperties::Named(vec!["metallic".to_owned()])
        );

        let options = resolve_over(
            &[
                "--dither",
                "none",
                "--partition",
                "ior",
                "--max-materials",
                "8",
            ],
            &retro,
        )
        .unwrap();
        assert_eq!(options.max_materials.get(), 8);
        assert_eq!(options.dither, Dither::None);
        assert_eq!(
            options.partition,
            PartitionProperties::Named(vec!["ior".to_owned()])
        );
    }
}
