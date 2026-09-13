use crate::{Result, cli_value_parser};
use clap::Args;
use meshconv::ReadFormat;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// The input mesh document and the format to read it as, shared by every
/// command that takes one.
#[derive(Clone, Debug, Args)]
pub struct MeshInput {
    /// The input mesh file, in any supported format.
    #[arg(value_name = "input")]
    pub path: PathBuf,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long, value_parser = cli_value_parser::<ReadFormat>())]
    from: Option<ReadFormat>,
}

impl MeshInput {
    /// The format to read the input as, `--from` or else the one the
    /// extension implies. Errors when neither gives one.
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
}
