use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document, vector3_i32,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxObject;
use voxsmith::operations::object::set_edit_bounds;

/// Sets objects' build volumes to a node-local box without moving a voxel in
/// the scene.
#[derive(Clone, Debug, Parser)]
#[command(name = "edit-bounds")]
pub struct ObjectSetEditBounds {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The box's min corner, as `node list --show-edit-bounds` prints it.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set
    )]
    min: Vec<i32>,

    /// The box's max corner, as `node list --show-edit-bounds` prints it.
    #[arg(
        value_names = ["x", "y", "z"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set
    )]
    max: Vec<i32>,
}

impl ObjectSetEditBounds {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let min = vector3_i32(&self.min);

        let max = vector3_i32(&self.max);

        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(set_edit_bounds(main, &object_ids, min, max)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectSetEditBounds;
    use clap::Parser;

    #[test]
    fn parses_both_corners() {
        let set = ObjectSetEditBounds::try_parse_from([
            "edit-bounds",
            "scene.voxj",
            "--select",
            "crate",
            "--min",
            "-2",
            "0",
            "0",
            "--max",
            "4",
            "8",
            "4",
        ])
        .unwrap();

        assert_eq!((set.min, set.max), (vec![-2, 0, 0], vec![4, 8, 4]));
    }
}
