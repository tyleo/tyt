use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use tyt_common::ExecFailed;

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    IO(IOError),
    Rg(ExecFailed),
    ScratchDirNotConfigured,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::ScratchDirNotConfigured => write!(
                f,
                "scratch_dir is not configured; add {{\"fs\": {{\"scratch_dir\": \"<path>\"}}}} to .tytconfig"
            ),
            Error::Rg(ExecFailed {
                exit_code,
                stdout,
                stderr,
            }) => {
                match exit_code {
                    Some(code) => write!(f, "rg exited with code {code}")?,
                    None => write!(f, "rg killed by signal")?,
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

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::Rg(_) | Error::ScratchDirNotConfigured => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
