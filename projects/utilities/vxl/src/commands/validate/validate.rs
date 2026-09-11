use crate::{Dependencies, Result, VoxelInput, cli_value_parser, file_name};
use clap::Parser;
use std::io::{Error as IOError, ErrorKind};
use voxconv::{check_document_files, read_document_files};
use voxcore::check;
use voxsmith::operations::validate::{ValidateLayout, validate};

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

        let files = read_document_files(&dependencies, from, &self.input.path)?;

        let checks = check_document_files(&dependencies, from, &files)?;

        let output = validate(&checks, &file_name(&self.input.path), self.layout);

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
