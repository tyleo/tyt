use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput,
    commands::parse_resample_factor, edit_document,
};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::{ResampleFactor, upsample_objects};

/// Splits each voxel of the objects into a block of a factor per axis: a
/// `10 x 10 x 10` object becomes `20 x 20 x 20` at factor `2`. The origin
/// scales with the grid. `node set scale` keeps the object's size in the
/// scene.
#[derive(Clone, Debug, Parser)]
#[command(name = "upsample")]
pub struct ObjectUpsample {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// How many voxels each voxel becomes per axis, at least `2`.
    #[arg(value_name = "factor", long, value_parser = parse_resample_factor)]
    factor: ResampleFactor,
}

impl ObjectUpsample {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(upsample_objects(main, &object_ids, self.factor)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectUpsample;
    use clap::{Error as ClapError, Parser};

    fn parse(factor: &str) -> Result<ObjectUpsample, ClapError> {
        ObjectUpsample::try_parse_from([
            "upsample",
            "scene.voxj",
            "--select",
            "crate",
            "--factor",
            factor,
        ])
    }

    #[test]
    fn parses_a_factor_of_at_least_two() {
        assert_eq!(parse("10").unwrap().factor.get(), 10);
        assert!(parse("1").is_err());
        assert!(
            ObjectUpsample::try_parse_from(["upsample", "scene.voxj", "--select", "crate"])
                .is_err()
        );
    }
}
