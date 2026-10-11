use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error decoding or encoding the payloads of a Voxel Max `.vmax` package.
#[derive(Debug)]
pub enum Error {
    /// The `scene.json` payload could not be parsed or written.
    Json(String),

    /// A binary property-list payload could not be parsed or serialized.
    Plist(String),

    /// A PNG payload could not be decoded or encoded.
    Png(String),

    /// Reading or writing a package file failed.
    Io(IOError),

    /// A payload was readable but semantically malformed.
    Invalid(String),

    /// Decoding the package file at the path failed with the inner error.
    File(String, Box<Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Json(message)
            | Error::Plist(message)
            | Error::Png(message)
            | Error::Invalid(message) => write!(f, "{message}"),

            Error::Io(e) => e.fmt(f),

            Error::File(path, error) => write!(f, "{path}: {error}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),

            Error::File(_, error) => Some(error.as_ref()),

            Error::Json(_) | Error::Plist(_) | Error::Png(_) | Error::Invalid(_) => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::Io(e)
    }
}
