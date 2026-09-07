use crate::{Result, cli_value_parser};
use clap::Args;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use voxconv::ReadFormat;

/// The input voxel document and the format to read it as, shared by every
/// command that takes one.
#[derive(Clone, Debug, Args)]
pub struct VoxelInput {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    pub path: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long, value_parser = cli_value_parser::<ReadFormat>())]
    from: Option<ReadFormat>,
}

impl VoxelInput {
    /// The format to read the input as: `--from` when given, else the format
    /// its extension implies. Errors when `--from` is absent and the
    /// extension implies no supported format.
    pub fn resolve_format(&self) -> Result<ReadFormat> {
        if let Some(format) = self.from {
            return Ok(format);
        }

        self.path
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(ReadFormat::from_extension)
            .ok_or_else(|| {
                IOError::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "could not infer the input format from `{}`; pass --from",
                        self.path.display()
                    ),
                )
                .into()
            })
    }

    /// `output` when given, else the input path with `extension`.
    pub fn output_path(&self, output: Option<PathBuf>, extension: &str) -> PathBuf {
        output.unwrap_or_else(|| self.path.with_extension(extension))
    }
}
