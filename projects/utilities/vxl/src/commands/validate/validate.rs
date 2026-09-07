use crate::{Dependencies, Result, VoxelInput, cli_value_parser, file_name};
use clap::Parser;
use std::io::{Error as IOError, ErrorKind};
use voxconv::{DependenciesImpl as VoxconvDependenciesImpl, codec};
use voxcore::check;
use voxsmith::{ValidateLayout, render_validation};

/// Checks a voxel document against its format's spec.
#[derive(Clone, Debug, Parser)]
#[command(name = "validate")]
pub struct Validate {
    #[command(flatten)]
    input: VoxelInput,

    /// How to lay out the report.
    #[arg(
        value_name = "layout",
        long,
        default_value = "tables",
        value_parser = cli_value_parser::<ValidateLayout>()
    )]
    layout: ValidateLayout,
}

impl Validate {
    /// Writes the per-check report to standard output, then fails when any
    /// check failed so the process exits non-zero.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let from = self.input.resolve_format()?;

        let files = codec::read_document_files(&dependencies, from, &self.input.path)?;

        let checks = codec::check_document_files(&VoxconvDependenciesImpl, from, &files)?;

        let output = render_validation(&checks, &file_name(&self.input.path), self.layout);

        dependencies.write_stdout(output.as_bytes())?;

        let failed = check::failed_check_count(&checks);

        if failed > 0 {
            // The report is already on stdout; exit non-zero with a terse summary.
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!("{failed} validation check{} failed", plural(failed)),
            )
            .into());
        }

        Ok(())
    }
}

/// The plural suffix for a count.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
