use crate::{
    Dependencies, ObjectSelection, Result, VoxelInput, cli_value_parser, file_name,
    warn_slashed_names,
};
use clap::Parser;
use voxconv::{
    ReadFormat,
    ext::read_with_ext,
    read_document_files,
    voxj::{ext::voxj_vox_ext_from_ext, voxj_version_from_bytes},
};
use voxsmith::operations::vox_doc::{VoxDocShowDocument, VoxDocShowLayout, vox_doc_show};

/// Prints what a document contains, surfacing the format internals.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct VoxDocShow {
    #[command(flatten)]
    input: VoxelInput,

    /// How to lay out the report.
    #[arg(
        value_name = "layout",
        long,
        default_value = "md-tables",
        value_parser = cli_value_parser::<VoxDocShowLayout>()
    )]
    layout: VoxDocShowLayout,

    #[command(flatten)]
    selection: ObjectSelection,
}

impl VoxDocShow {
    /// The document is read into voxcore first. Only the format and, for
    /// Voxel Json, the document version come from outside that model.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let files = read_document_files(&dependencies, from, &self.input.path)?;

        // The boxed ext keeps any source's ext, so the report can say whether
        // the document carries entries.
        let main = read_with_ext(&dependencies, from, &files)?;

        warn_slashed_names(&dependencies, main.state())?;

        let format_version = match from {
            ReadFormat::Voxj => {
                let file = files
                    .first()
                    .expect("the read accepted a single-file document");

                Some(voxj_version_from_bytes(&dependencies, &file.bytes)?)
            }

            _ => None,
        };

        let name = file_name(&self.input.path);

        let document = VoxDocShowDocument {
            name: &name,
            format: from.name(),
            format_version,
            has_ext: !voxj_vox_ext_from_ext(main.ext().as_ref())?.is_empty(),
        };

        let object_ids = self.selection.resolve(&main)?;

        let output = vox_doc_show(&main, &object_ids, &document, self.layout);

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}
