use crate::{Result, implementation};
use std::{
    fs,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxsmith::{ValidateLayout, check_voxj_bytes, failed_check_count, render_validation};

/// Loads the Voxel Json document at `input`, runs every spec check, writes the
/// report in `layout` to standard output, and fails when any check failed so
/// the process exits non-zero.
pub fn validate(input: &Path, layout: ValidateLayout) -> Result<()> {
    let checks = check_voxj_bytes(&fs::read(input)?)?;

    let output = render_validation(&checks, &file_name(input), layout);

    implementation::write_stdout(output.as_bytes())?;

    let failed = failed_check_count(&checks);

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

/// The input's file name for the report heading, or its full path when it has
/// none.
fn file_name(input: &Path) -> String {
    input
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| input.display().to_string())
}

/// The plural suffix for a count.
fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
