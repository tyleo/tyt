use crate::{ExecError, exec};
use std::ffi::OsStr;
use tyt_common::ExecFailed;

/// Executes an external command, mapping errors through the provided constructors.
pub fn exec_map<I, S, E>(
    program: &str,
    args: I,
    map_io: impl FnOnce(std::io::Error) -> E,
    map_failed: impl FnOnce(ExecFailed) -> E,
) -> std::result::Result<Vec<u8>, E>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    exec(program, args).map_err(|e| match e {
        ExecError::IO(e) => map_io(e),
        ExecError::Failed(f) => map_failed(f),
    })
}
