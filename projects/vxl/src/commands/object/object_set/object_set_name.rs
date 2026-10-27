use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::Error as VoxsmithError;

/// Renames one object.
#[derive(Clone, Debug, Parser)]
#[command(name = "name")]
pub struct ObjectSetName {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The object's new name.
    #[arg(value_name = "name", long)]
    name: String,
}

impl ObjectSetName {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_id = self.selection.resolve_one_object(main)?;

            Ok(main
                .set_object_name(object_id, self.name)
                .map_err(VoxsmithError::from)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectSetName;
    use clap::Parser;

    #[test]
    fn parses_the_name() {
        let set = ObjectSetName::try_parse_from([
            "name",
            "scene.voxj",
            "--select",
            "crate",
            "--name",
            "box",
        ])
        .unwrap();

        assert_eq!(set.name, "box");
    }
}
