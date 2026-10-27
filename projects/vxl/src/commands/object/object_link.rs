use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document,
};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::link_objects;

/// Places objects under one more parent node. Errors when the parent already
/// places one.
#[derive(Clone, Debug, Parser)]
#[command(name = "link")]
pub struct ObjectLink {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl ObjectLink {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            let parent_id = self.parent.resolve_required(main)?;

            Ok(link_objects(main, &object_ids, parent_id)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectLink;
    use clap::Parser;

    #[test]
    fn parses_the_selectors_and_the_parent() {
        let link = ObjectLink::try_parse_from([
            "link",
            "scene.voxj",
            "--select",
            "door",
            "--select-parent",
            "house",
        ])
        .unwrap();

        assert_eq!(link.input.path.to_str(), Some("scene.voxj"));
        assert!(
            ObjectLink::try_parse_from(["link", "scene.voxj", "--select-parent", "house"]).is_err()
        );
    }
}
