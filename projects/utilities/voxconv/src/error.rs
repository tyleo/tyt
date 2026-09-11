use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from voxconv.
#[derive(Debug)]
pub enum Error {
    /// A document's files do not fit its format: a single-file format given
    /// more or fewer than one file, or a package entry without a UTF-8 path.
    Files(String),

    /// Reading or writing a document's files failed.
    Io(IOError),

    /// An ext failed to encode to or decode from its Voxel Json slots.
    Ext(String),

    /// A format's bridge failed to convert a document. Each format module
    /// converts its bridge's error into this.
    Format(Box<dyn StdError + Send + Sync>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Files(message) => write!(f, "{message}"),
            Error::Io(error) => error.fmt(f),
            Error::Ext(message) => write!(f, "{message}"),
            Error::Format(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Files(_) => None,
            Error::Io(error) => Some(error),
            Error::Ext(_) => None,
            Error::Format(error) => Some(error.as_ref()),
        }
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error::Io(error)
    }
}
