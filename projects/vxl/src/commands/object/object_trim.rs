use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::trim_objects;

/// Shrinks objects to their live voxels without moving a voxel in the scene.
#[derive(Clone, Debug, Parser)]
#[command(name = "trim")]
pub struct ObjectTrim {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,
}

impl ObjectTrim {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(trim_objects(main, &object_ids)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectTrim;
    use clap::Parser;

    #[test]
    fn parses_the_selectors() {
        assert!(ObjectTrim::try_parse_from(["trim", "scene.voxj", "--select-index", "0"]).is_ok());
        assert!(ObjectTrim::try_parse_from(["trim", "scene.voxj"]).is_err());
    }
}
