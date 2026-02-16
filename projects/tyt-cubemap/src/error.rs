use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from a cubemap operation.
#[derive(Debug)]
pub enum Error {
    Ffmpeg {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    Magick {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    IO(IOError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Ffmpeg {
                exit_code,
                stdout,
                stderr,
            } => {
                match exit_code {
                    Some(code) => write!(f, "ffmpeg exited with code {code}")?,
                    None => write!(f, "ffmpeg killed by signal")?,
                }
                if !stdout.is_empty() {
                    write!(f, "\nstdout:\n{stdout}")?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nstderr:\n{stderr}")?;
                }
                Ok(())
            }
            Error::Magick {
                exit_code,
                stdout,
                stderr,
            } => {
                match exit_code {
                    Some(code) => write!(f, "magick exited with code {code}")?,
                    None => write!(f, "magick killed by signal")?,
                }
                if !stdout.is_empty() {
                    write!(f, "\nstdout:\n{stdout}")?;
                }
                if !stderr.is_empty() {
                    write!(f, "\nstderr:\n{stderr}")?;
                }
                Ok(())
            }
            Error::IO(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Ffmpeg { .. } | Error::Magick { .. } => None,
            Error::IO(e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
