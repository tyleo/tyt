use crate::{Dependencies, Result, VoxelInput, cli_value_parser, file_name};
use clap::Parser;
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, ReadFormat, codec};
use voxcore::{VoxMain, VoxMap};
use voxsmith::{InfoDocument, InfoLayout, render_info};

/// Reports what a document contains, surfacing the format internals.
#[derive(Clone, Debug, Parser)]
#[command(name = "info")]
pub struct Info {
    #[command(flatten)]
    input: VoxelInput,

    /// How to lay out the report.
    #[arg(
        value_name = "layout",
        long,
        default_value = "tables",
        value_parser = cli_value_parser::<InfoLayout>()
    )]
    layout: InfoLayout,
}

impl Info {
    /// The document is read into voxcore first. Only the format and, for
    /// Voxel Json, the document version come from outside that model.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let files = codec::read_document_files(&dependencies, from, &self.input.path)?;

        // The raw slot keeps any source's ext as a block, so the report can say
        // whether the document carries one.
        let state: VoxMain<Option<VoxMap>> = codec::read(&VoxconvDependenciesImpl, from, &files)?;

        let format_version = match from {
            ReadFormat::Voxj => {
                let file = files
                    .first()
                    .expect("the read accepted a single-file document");

                Some(codec::voxj_version_from_bytes(
                    &VoxconvDependenciesImpl,
                    &file.bytes,
                )?)
            }
            _ => None,
        };

        let name = file_name(&self.input.path);

        let document = InfoDocument {
            name: &name,
            format: from.name(),
            format_version,
            has_ext: state.ext().is_some(),
        };

        let output = render_info(&state, &document, self.layout);

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}
