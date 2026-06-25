use crate::{ExecError, exec_ignore_stderr};
use std::{ffi::OsStr, io::Error as IOError, result::Result as StdResult};
use tyt_common::ExecFailed;

/// Executes an external command, mapping errors through the provided
/// constructors. Only a non-zero exit code is treated as failure; stderr
/// output is ignored.
pub fn exec_map_ignore_stderr<I, S, E>(
    program: &str,
    args: I,
    map_io: impl FnOnce(IOError) -> E,
    map_failed: impl FnOnce(ExecFailed) -> E,
) -> StdResult<Vec<u8>, E>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec_ignore_stderr(program, args).map_err(|e| match e {
        ExecError::IO(e) => map_io(e),
        ExecError::Failed(f) => map_failed(f),
    })
}
