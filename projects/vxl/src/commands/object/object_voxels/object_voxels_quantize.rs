use crate::{
    Dependencies, ObjectSelection, QuantizeArgs, QuantizeProfile, Result, VoxelInput, VoxjOutput,
    commands::load_object_voxels_quantize_profile_set, edit_document, parse_id_selector,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxLayer;
use voxsmith::{operations::object::quantize_object_voxels, utilities::IdSelector};

/// Rewrites objects' voxel samples so each quantized layer samples at most
/// `--max-materials` materials of its palette. Palettes and value pools stay
/// untouched. `palette quantize` reduces the palette itself.
#[derive(Clone, Debug, Parser)]
#[command(name = "quantize")]
pub struct ObjectVoxelsQuantize {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: ObjectSelection,

    /// Quantizes each selected object's layers by id: an integer, an inclusive
    /// `a-b` range, or `*` for every layer. A named layer an object lacks
    /// errors. When omitted, every layer whose palette binds `--property` is
    /// quantized. Repeatable.
    #[arg(value_name = "layer-index", long, value_parser = parse_id_selector::<BVoxLayer>)]
    layer_index: Vec<IdSelector<BVoxLayer>>,

    /// Clusters every selected layer on one palette together, where otherwise
    /// each object clusters apart.
    #[arg(
        value_name = "shared",
        long,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set
    )]
    shared: bool,

    /// Applies saved quantize flags. A flag given here overrides the element
    /// it mirrors. The profiles come from every
    /// `.vxlconfig`'s `object.voxels.quantize.profiles`, the user's
    /// `~/.vxlconfig` first and then each directory from the git root down to
    /// the working directory. A name reads from the last file supplying it.
    #[arg(value_name = "profile", long)]
    profile: Option<String>,

    #[command(flatten)]
    quantize: QuantizeArgs,
}

impl ObjectVoxelsQuantize {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let profile = match &self.profile {
            Some(name) => load_object_voxels_quantize_profile_set(&dependencies)?
                .get("--profile", name)?
                .clone(),

            None => QuantizeProfile::default(),
        };
        let options = self.quantize.resolve(&profile)?;

        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve(main)?;

            Ok(quantize_object_voxels(
                main,
                &object_ids,
                &self.layer_index,
                self.shared,
                &options,
            )?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectVoxelsQuantize;
    use clap::{Error as ClapError, Parser};

    fn parse(args: &[&str]) -> Result<ObjectVoxelsQuantize, ClapError> {
        let mut argv = vec!["quantize", "scene.voxj", "--max-materials", "16"];
        argv.extend_from_slice(args);
        ObjectVoxelsQuantize::try_parse_from(argv)
    }

    #[test]
    fn parses_layer_indices_and_shared() {
        let quantize = parse(&["--layer-index", "0", "--layer-index", "2-3", "--shared"]).unwrap();

        assert_eq!(quantize.layer_index.len(), 2);
        assert!(quantize.shared);
        assert!(!parse(&[]).unwrap().shared);
        assert!(parse(&["--layer-index", "3-2"]).is_err());
    }
}
