use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "impl")]
    Glob(globset::Error),
    IO(IOError),
    #[cfg(feature = "impl")]
    Json(serde_json::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            #[cfg(feature = "impl")]
            Error::Glob(e) => e.fmt(f),
            Error::IO(e) => e.fmt(f),
            #[cfg(feature = "impl")]
            Error::Json(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "impl")]
            Error::Glob(e) => Some(e),
            Error::IO(e) => Some(e),
            #[cfg(feature = "impl")]
            Error::Json(e) => Some(e),
        }
    }
}

#[cfg(feature = "impl")]
impl From<globset::Error> for Error {
    fn from(e: globset::Error) -> Self {
        Error::Glob(e)
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}

#[cfg(feature = "impl")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}
