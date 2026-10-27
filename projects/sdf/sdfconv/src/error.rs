use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from sdfconv.
#[derive(Debug)]
pub enum Error {
    /// A format's bridge failed to convert a document. Each format module
    /// converts its bridge's error into this.
    Format(Box<dyn StdError + Send + Sync>),

    /// Reading or writing a document's file failed.
    Io(IOError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Format(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Format(error) => Some(error.as_ref()),
            Error::Io(error) => Some(error),
        }
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error::Io(error)
    }
}
