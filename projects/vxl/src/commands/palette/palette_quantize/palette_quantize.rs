use crate::{
    Dependencies, QuantizeArgs, QuantizeProfile, Result, VoxelInput, VoxjOutput,
    commands::load_palette_quantize_profile_set, edit_document, parse_id_selector,
};
use clap::Parser;
use voxcore::BVoxPalette;
use voxsmith::{
    operations::palette::quantize_palette,
    utilities::{IdSelector, resolve_palette_selectors},
};

/// Reduces each selected palette to at most `--max-materials` materials and
/// snaps every layer referencing it. Materials no voxel samples drop, and the
/// rest compact. A selected palette no live voxel samples errors.
/// `object voxels quantize` reduces what objects sample without changing the
/// palette.
#[derive(Clone, Debug, Parser)]
#[command(name = "quantize")]
pub struct PaletteQuantize {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    /// Which palettes to quantize: an integer, an inclusive `a-b` range, or `*`
    /// for every palette. Repeatable; the union selects each palette once.
    #[arg(
        value_name = "palette",
        long,
        value_parser = parse_id_selector::<BVoxPalette>,
        default_value = "*"
    )]
    index: Vec<IdSelector<BVoxPalette>>,

    /// Applies saved quantize flags. A flag given here overrides the element
    /// it mirrors. The profiles come from every
    /// `.vxlconfig`'s `palette.quantize.profiles`, the user's `~/.vxlconfig`
    /// first and then each directory from the git root down to the working
    /// directory. A name reads from the last file supplying it.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,

    #[command(flatten)]
    quantize: QuantizeArgs,
}

impl PaletteQuantize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profile = match &self.profile {
            Some(name) => load_palette_quantize_profile_set(&dependencies)?
                .get("--profile", name)?
                .clone(),

            None => QuantizeProfile::default(),
        };
        let options = self.quantize.resolve(&profile)?;

        edit_document(&dependencies, &self.input, self.output, |main| {
            let palette_ids = resolve_palette_selectors(main, &self.index)?;

            Ok(quantize_palette(main, &palette_ids, &options)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PaletteQuantize;
    use branded_id::U32Id;
    use clap::Parser;
    use voxsmith::utilities::IdSelector;

    #[test]
    fn index_defaults_to_every_palette_and_repeats() {
        let parse = |args: &[&str]| {
            let mut argv = vec!["quantize", "scene.voxj", "--max-materials", "16"];
            argv.extend_from_slice(args);
            PaletteQuantize::try_parse_from(argv).unwrap()
        };

        assert_eq!(parse(&[]).index, [IdSelector::all()]);

        assert_eq!(
            parse(&["--index", "2", "--index", "0-1"]).index,
            [
                IdSelector::id(U32Id::from_u32(2)),
                IdSelector::range(U32Id::from_u32(0)..=U32Id::from_u32(1)).unwrap(),
            ]
        );
    }
}
