use crate::{Format, Result};
use std::{
    io::{Error as IOError, ErrorKind},
    path::Path,
};

/// The source format to read `input` as: `from` when given, else inferred
/// from the path's extension. Errors when `from` is absent and the extension
/// matches no supported format.
pub fn resolve_format(input: &Path, from: Option<Format>) -> Result<Format> {
    match from {
        Some(format) => Ok(format),
        None => Format::from_path(input).ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidInput,
                format!(
                    "could not infer the input format from `{}`; pass --from",
                    input.display()
                ),
            )
            .into()
        }),
    }
}
