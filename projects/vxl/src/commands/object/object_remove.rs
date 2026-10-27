use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::operations::object::remove_objects;

/// Removes objects, then each node the removal left childless.
#[derive(Clone, Debug, Parser)]
#[command(name = "remove")]
pub struct ObjectRemove {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,
}

impl ObjectRemove {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_ids = self.selection.resolve_objects(main)?;

            Ok(remove_objects(main, &object_ids)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectRemove;
    use clap::Parser;

    #[test]
    fn parses_the_input_output_and_selectors() {
        let remove = ObjectRemove::try_parse_from([
            "remove",
            "scene.vmax",
            "out.voxjz",
            "--select",
            "debris/**",
            "--select-index",
            "0-2",
        ])
        .unwrap();

        assert_eq!(remove.input.path.to_str(), Some("scene.vmax"));
        assert!(ObjectRemove::try_parse_from(["remove", "scene.vmax"]).is_err());
    }
}
