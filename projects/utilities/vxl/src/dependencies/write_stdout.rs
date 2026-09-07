use std::io::Result as IOResult;

/// Writes to standard output.
pub trait WriteStdout {
    /// Writes `contents` to standard output.
    fn write_stdout(&self, contents: &[u8]) -> IOResult<()>;
}
