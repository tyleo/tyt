use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, cli_value_parser,
    edit_document,
};
use clap::Parser;
use ty_math::TyAxis3;
use voxcore::BVoxObject;
use voxsmith::operations::object::{QuarterTurns, rotate_object_voxels};

/// Turns objects' voxels about the grid's center in quarter turns that follow
/// the right-hand rule. One or three turns need the two turned dimensions to be
/// equal. `node set rotation` turns any object.
#[derive(Clone, Debug, Parser)]
#[command(name = "rotate")]
pub struct ObjectVoxelsRotate {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The grid axis to turn about.
    #[arg(value_name = "axis", long, value_parser = cli_value_parser::<TyAxis3>())]
    axis: TyAxis3,

    /// How many quarter turns to make.
    #[arg(value_name = "turns", long, value_parser = cli_value_parser::<QuarterTurns>())]
    turns: QuarterTurns,
}

impl ObjectVoxelsRotate {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(rotate_object_voxels(
                main,
                &object_ids,
                self.axis,
                self.turns,
            )?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectVoxelsRotate;
    use clap::{Error as ClapError, Parser};
    use ty_math::TyAxis3;
    use voxsmith::operations::object::QuarterTurns;

    fn parse(turns: &str) -> Result<ObjectVoxelsRotate, ClapError> {
        ObjectVoxelsRotate::try_parse_from([
            "rotate",
            "scene.voxj",
            "--select",
            "crate",
            "--axis",
            "y",
            "--turns",
            turns,
        ])
    }

    #[test]
    fn parses_one_to_three_turns() {
        let rotate = parse("1").unwrap();

        assert_eq!(rotate.axis, TyAxis3::Y);
        assert_eq!(rotate.turns, QuarterTurns::One);
        assert_eq!(parse("3").unwrap().turns, QuarterTurns::Three);
        assert!(parse("0").is_err());
        assert!(parse("4").is_err());
    }
}
