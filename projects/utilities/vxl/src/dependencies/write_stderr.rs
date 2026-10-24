use std::io::Result as IOResult;

/// Writes to standard error.
pub trait WriteStderr {
    /// Writes `contents` to standard error.
    fn write_stderr(&self, contents: &[u8]) -> IOResult<()>;
}
