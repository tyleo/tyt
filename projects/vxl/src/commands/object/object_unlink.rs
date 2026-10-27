use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::unlink_objects;

/// Removes objects from one parent node, leaving an object with no parents
/// unplaced. Errors when the parent does not place one.
#[derive(Clone, Debug, Parser)]
#[command(name = "unlink")]
pub struct ObjectUnlink {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl ObjectUnlink {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            let parent_id = self.parent.resolve_required(main)?;

            Ok(unlink_objects(main, &object_ids, parent_id)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectUnlink;
    use clap::Parser;

    #[test]
    fn parses_the_selectors_and_the_parent() {
        let unlink = ObjectUnlink::try_parse_from([
            "unlink",
            "scene.voxj",
            "--select",
            "door",
            "--select-parent",
            "house",
        ])
        .unwrap();

        assert_eq!(unlink.input.path.to_str(), Some("scene.voxj"));
        assert!(
            ObjectUnlink::try_parse_from(["unlink", "scene.voxj", "--select-parent", "house"])
                .is_err()
        );
    }
}
