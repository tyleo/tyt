use crate::{Dependencies, RequiredSelection, Result, VoxelInput, VoxjOutput, edit_document};
use clap::Parser;
use voxcore::BVoxObject;
use voxsmith::Error as VoxsmithError;

/// Moves one object to a position in the object list.
#[derive(Clone, Debug, Parser)]
#[command(name = "reorder")]
pub struct ObjectReorder {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    /// The position to move the object to, in the object list that
    /// `--select-index` counts.
    #[arg(value_name = "index", long)]
    index: usize,
}

impl ObjectReorder {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        edit_document(&dependencies, &self.input, self.output, |main| {
            let object_id = self.selection.resolve_one_object(main)?;

            Ok(main
                .move_object(object_id, self.index)
                .map_err(VoxsmithError::from)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectReorder;
    use clap::Parser;

    #[test]
    fn parses_the_index() {
        let reorder = ObjectReorder::try_parse_from([
            "reorder",
            "scene.voxj",
            "--select",
            "crate",
            "--index",
            "2",
        ])
        .unwrap();

        assert_eq!(reorder.index, 2);
        assert!(
            ObjectReorder::try_parse_from(["reorder", "scene.voxj", "--select", "crate"]).is_err()
        );
    }
}
