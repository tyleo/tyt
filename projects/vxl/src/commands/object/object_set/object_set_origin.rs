use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document, vector3_i32,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxObject;
use voxsmith::Error as VoxsmithError;

/// Moves objects' grids, and their voxels with them, relative to the placing
/// node.
#[derive(Clone, Debug, Parser)]
#[command(name = "origin")]
pub struct ObjectSetOrigin {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The grid's new min corner relative to the placing node.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set
    )]
    origin: Vec<i32>,
}

impl ObjectSetOrigin {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let origin = vector3_i32(&self.origin);

        edit_document(&dependencies, &self.input, self.output, |main| {
            for object_id in self.selection.resolve_objects(main)? {
                main.set_object_origin(object_id, origin)
                    .map_err(VoxsmithError::from)?;
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectSetOrigin;
    use clap::Parser;

    #[test]
    fn parses_a_negative_origin() {
        let set = ObjectSetOrigin::try_parse_from([
            "origin",
            "scene.voxj",
            "--select",
            "crate",
            "--origin",
            "-2",
            "0",
            "3",
            "out.voxj",
        ])
        .unwrap();

        assert_eq!(set.origin, [-2, 0, 3]);
        assert!(
            ObjectSetOrigin::try_parse_from([
                "origin",
                "scene.voxj",
                "--select",
                "crate",
                "--origin",
                "1",
                "2"
            ])
            .is_err()
        );
    }
}
