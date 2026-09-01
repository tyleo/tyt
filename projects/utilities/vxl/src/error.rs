use clap::{Error as ClapError, error::ErrorKind};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

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

#[cfg(feature = "impl")]
impl From<vmax_codec::Error> for Error {
    fn from(e: vmax_codec::Error) -> Self {
        Error::IO(IOError::other(e))
    }
}

#[cfg(feature = "impl")]
impl From<voxcore::ext::Error> for Error {
    fn from(e: voxcore::ext::Error) -> Self {
        Error::IO(IOError::other(e))
    }
}

#[cfg(feature = "impl")]
impl From<voxj_codec::Error> for Error {
    fn from(e: voxj_codec::Error) -> Self {
        Error::IO(IOError::other(e))
    }
}

#[cfg(feature = "impl")]
impl From<voxj_voxcore::Error> for Error {
    fn from(e: voxj_voxcore::Error) -> Self {
        Error::IO(IOError::other(e))
    }
}

#[cfg(feature = "impl")]
impl From<voxsmith::Error> for Error {
    fn from(e: voxsmith::Error) -> Self {
        Error::IO(IOError::other(e))
    }
}
