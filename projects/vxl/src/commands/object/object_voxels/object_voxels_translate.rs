use crate::{
    Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document, vector3_i32,
};
use clap::{ArgAction, Parser};
use voxcore::BVoxObject;
use voxsmith::operations::object::translate_object_voxels;

/// Moves objects' voxels within their grids. Errors when a voxel would leave
/// its grid.
#[derive(Clone, Debug, Parser)]
#[command(name = "translate")]
pub struct ObjectVoxelsTranslate {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// How far to move each voxel, in voxels.
    #[arg(
        value_names = ["dx", "dy", "dz"],
        long,
        num_args = 3,
        required = true,
        allow_negative_numbers = true,
        action = ArgAction::Set
    )]
    offset: Vec<i32>,
}

impl ObjectVoxelsTranslate {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let offset = vector3_i32(&self.offset);

        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(translate_object_voxels(main, &object_ids, offset)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectVoxelsTranslate;
    use clap::Parser;

    #[test]
    fn parses_a_negative_offset() {
        let translate = ObjectVoxelsTranslate::try_parse_from([
            "translate",
            "scene.voxj",
            "--select",
            "crate",
            "--offset",
            "-1",
            "0",
            "2",
        ])
        .unwrap();

        assert_eq!(translate.offset, [-1, 0, 2]);
        assert!(
            ObjectVoxelsTranslate::try_parse_from([
                "translate",
                "scene.voxj",
                "--select",
                "crate",
                "--offset",
                "1"
            ])
            .is_err()
        );
    }
}
