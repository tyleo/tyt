use crate::ExecError;
use std::{ffi::OsStr, process};

/// Executes an external command and returns its stdout on success.
pub fn exec<I, S>(program: &str, args: I) -> std::result::Result<Vec<u8>, ExecError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = process::Command::new(program)
        .args(args)
        .output()
        .map_err(ExecError::IO)?;

    if !output.status.success() || !output.stderr.is_empty() {
        return Err(ExecError::Failed {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    Ok(output.stdout)
}
