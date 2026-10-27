use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, cli_value_parser,
    commands::parse_resample_factor, edit_document,
};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::{KeepRule, ResampleFactor, downsample_objects};

/// Merges objects' voxels into blocks of a factor per axis: a `10 x 10 x 10`
/// object becomes `5 x 5 x 5` at factor `2`. The blocks tile the node's
/// lattice, which grows the grid to cover an origin the factor does not
/// divide. `node set scale` keeps the object's size in the scene.
#[derive(Clone, Debug, Parser)]
#[command(name = "downsample")]
pub struct ObjectDownsample {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// How many voxels per axis merge into one, at least `2`.
    #[arg(value_name = "factor", long, value_parser = parse_resample_factor)]
    factor: ResampleFactor,

    /// When a block is kept live, by how many of its cells are. A kept block
    /// takes the material most of its live cells share.
    #[arg(
        value_name = "keep",
        long,
        default_value = "majority",
        value_parser = cli_value_parser::<KeepRule>()
    )]
    keep: KeepRule,
}

impl ObjectDownsample {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(downsample_objects(
                main,
                &object_ids,
                self.factor,
                self.keep,
            )?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectDownsample;
    use clap::{Error as ClapError, Parser};
    use voxsmith::operations::object::KeepRule;

    fn parse(extra: &[&str]) -> Result<ObjectDownsample, ClapError> {
        let mut argv = vec!["downsample", "scene.voxj", "--select", "crate"];
        argv.extend_from_slice(extra);
        ObjectDownsample::try_parse_from(argv)
    }

    #[test]
    fn parses_the_factor_and_defaults_the_keep_rule() {
        let downsample = parse(&["--factor", "2"]).unwrap();

        assert_eq!(downsample.factor.get(), 2);
        assert_eq!(downsample.keep, KeepRule::Majority);
        assert_eq!(
            parse(&["--factor", "3", "--keep", "any"]).unwrap().keep,
            KeepRule::Any
        );
        assert!(parse(&["--factor", "1"]).is_err());
        assert!(parse(&["--factor", "2", "--keep", "most"]).is_err());
        assert!(parse(&[]).is_err());
    }
}
