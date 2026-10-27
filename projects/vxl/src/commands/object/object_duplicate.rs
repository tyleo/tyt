use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::duplicate_objects;

/// Appends a copy of each object with the original's name and palettes. Every
/// parent of the original also places the copy, unless a parent selector gives
/// one node instead.
#[derive(Clone, Debug, Parser)]
#[command(name = "duplicate")]
pub struct ObjectDuplicate {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl ObjectDuplicate {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            let parent_id = self.parent.resolve(main)?;

            duplicate_objects(main, &object_ids, parent_id)?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectDuplicate;
    use clap::Parser;

    #[test]
    fn parses_the_selectors_with_an_optional_parent() {
        let duplicate =
            ObjectDuplicate::try_parse_from(["duplicate", "scene.voxj", "--select-index", "0"])
                .unwrap();

        assert_eq!(duplicate.input.path.to_str(), Some("scene.voxj"));
        assert!(
            ObjectDuplicate::try_parse_from([
                "duplicate",
                "scene.voxj",
                "--select-index",
                "0",
                "--select-parent-index",
                "2",
            ])
            .is_ok()
        );
    }
}
