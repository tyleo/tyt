use clap::{Error as ClapError, error::ErrorKind};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use voxconv::Error as VoxconvError;
use voxsmith::Error as VoxsmithError;

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    IO(IOError),

    /// A usage error, rendered and exited like a clap parse failure.
    Usage(ClapError),
}

impl Error {
    /// A usage error for a rule clap cannot express, presented like a clap parse
    /// failure: clap's formatting and exit code 2.
    pub(crate) fn usage(message: impl Display) -> Error {
        Error::Usage(ClapError::raw(
            ErrorKind::ValueValidation,
            format!("{message}\n"),
        ))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::Usage(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::Usage(e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}

impl From<VoxconvError> for Error {
    fn from(e: VoxconvError) -> Self {
        Error::IO(IOError::other(e))
    }
}

impl From<VoxsmithError> for Error {
    fn from(e: VoxsmithError) -> Self {
        Error::IO(IOError::other(e))
    }
}
