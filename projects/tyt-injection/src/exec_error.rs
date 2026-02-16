use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use tyt_common::ExecFailed;

/// An error from executing an external command.
#[derive(Debug)]
pub enum ExecError {
    IO(IOError),
    Failed(ExecFailed),
}

impl Display for ExecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ExecError::IO(e) => e.fmt(f),
            ExecError::Failed(ExecFailed {
                exit_code,
                stdout,
                stderr,
            }) => {
                match exit_code {
                    Some(code) => write!(f, "command exited with code {code}")?,
                    None => write!(f, "command killed by signal")?,
                }
                if !stdout.is_empty() {
                    write!(f, "\nstdout:\n{stdout}")?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nstderr:\n{stderr}")?;
                }
                Ok(())
            }
        }
    }
}

impl StdError for ExecError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ExecError::IO(e) => Some(e),
            ExecError::Failed(_) => None,
        }
    }
}
