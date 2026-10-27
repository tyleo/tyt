use std::{ffi::OsString, io::Result as IOResult};

/// Runs other programs.
pub trait RunProgram {
    /// Runs `program` with `args` on this process's standard streams and waits
    /// for it to exit. Returns the exit code, or `None` when a signal ended the
    /// program.
    fn run_program(&self, program: &str, args: &[OsString]) -> IOResult<Option<i32>>;
}
