use crate::{ExecError, exec};
use std::{ffi::OsStr, io::Error as IOError, result::Result as StdResult};
use tyt_common::ExecFailed;

/// Executes an external command, mapping errors through the provided constructors.
pub fn exec_map<I, S, E>(
    program: &str,
    args: I,
    map_io: impl FnOnce(IOError) -> E,
    map_failed: impl FnOnce(ExecFailed) -> E,
) -> StdResult<Vec<u8>, E>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec(program, args).map_err(|e| match e {
        ExecError::IO(e) => map_io(e),
        ExecError::Failed(f) => map_failed(f),
    })
}
