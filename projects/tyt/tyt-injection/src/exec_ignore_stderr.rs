use crate::ExecError;
use std::{ffi::OsStr, process, result::Result as StdResult};
use tyt_common::ExecFailed;

/// Executes an external command and returns its stdout on success, ignoring
/// stderr output. Only a non-zero exit code is treated as failure.
pub fn exec_ignore_stderr<I, S>(program: &str, args: I) -> StdResult<Vec<u8>, ExecError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = process::Command::new(program)
        .args(args)
        .output()
        .map_err(ExecError::IO)?;

    if !output.status.success() {
        return Err(ExecError::Failed(ExecFailed {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }));
    }

    Ok(output.stdout)
}
