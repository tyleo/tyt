use globset::Error as GlobError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error compiling a gitignore glob pattern.
#[derive(Debug)]
pub enum Error {
    /// The pattern is not a valid glob.
    Glob(GlobError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Glob(error) => write!(formatter, "{error}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Glob(error) => Some(error),
        }
    }
}

impl From<GlobError> for Error {
    fn from(error: GlobError) -> Self {
        Error::Glob(error)
    }
}
