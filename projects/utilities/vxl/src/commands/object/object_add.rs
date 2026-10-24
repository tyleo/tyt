use crate::{
    Dependencies, ParentSelection, RequiredSelection, Result, VoxelInput, VoxjOutput,
    cli_value_parser, edit_document, load_vox_main,
};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use voxconv::ReadFormat;
use voxcore::{BVoxObject, VoxMain};
use voxsmith::operations::object::add_objects;

/// Copies objects from another document. `--select` and `--select-index`
/// choose them in the source. Each source palette a copy references is
/// appended once with its value pools. Each copy goes under the parent node,
/// or else under a new root node named for it. The source's `ext` block is
/// not carried.
#[derive(Clone, Debug, Parser)]
#[command(name = "add")]
pub struct ObjectAdd {
    #[command(flatten)]
    input: VoxelInput,

    #[command(flatten)]
    output: VoxjOutput,

    /// The voxel file to copy objects from, in any supported format.
    #[arg(value_name = "source", long)]
    source: PathBuf,

    /// Format of the source. Defaults to the format its extension implies.
    #[arg(value_name = "source-from", long, value_parser = cli_value_parser::<ReadFormat>())]
    source_from: Option<ReadFormat>,

    #[command(flatten)]
    selection: RequiredSelection<BVoxObject>,

    #[command(flatten)]
    parent: ParentSelection,
}

impl ObjectAdd {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let source_from = self.source_format()?;

        let source: VoxMain = load_vox_main(&dependencies, source_from, &self.source)?;

        let object_ids = self.selection.resolve_objects(&source)?;

        edit_document(&dependencies, &self.input, self.output, |main| {
            let parent_id = self.parent.resolve(main)?;

            add_objects(main, source.state(), &object_ids, parent_id)?;

            Ok(())
        })
    }

    fn source_format(&self) -> Result<ReadFormat> {
        if let Some(format) = self.source_from {
            return Ok(format);
        }

        self.source
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(ReadFormat::from_extension)
            .ok_or_else(|| {
                IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "could not infer the source format from `{}`; pass --source-from",
                        self.source.display()
                    ),
                )
                .into()
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::ObjectAdd;
    use clap::Parser;

    #[test]
    fn parses_the_source_selectors_and_parent() {
        let add = ObjectAdd::try_parse_from([
            "add",
            "scene.voxj",
            "--source",
            "props.voxj",
            "--select-index",
            "0-2",
            "--select-parent",
            "house",
        ])
        .unwrap();

        assert_eq!(add.source.to_str(), Some("props.voxj"));
        assert!(add.source_format().is_ok());
        assert!(ObjectAdd::try_parse_from(["add", "scene.voxj", "--select-index", "0"]).is_err());
    }

    #[test]
    fn an_unknown_source_extension_needs_source_from() {
        let add = ObjectAdd::try_parse_from([
            "add",
            "scene.voxj",
            "--source",
            "props.bin",
            "--select-index",
            "0",
        ])
        .unwrap();

        assert!(add.source_format().is_err());
    }
}
