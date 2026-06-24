use plist::Error as PlistError;
use png::{DecodingError, EncodingError};
use serde_json::Error as JsonError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error decoding or encoding the payloads of a Voxel Max `.vmax` package.
#[derive(Debug)]
pub enum Error {
    /// A JSON payload (for example `scene.json`) could not be deserialized or
    /// serialized.
    Json(JsonError),

    /// A binary property-list payload could not be deserialized or serialized.
    Plist(PlistError),

    /// A PNG payload could not be decoded.
    PngDecode(DecodingError),

    /// A PNG payload could not be encoded.
    PngEncode(EncodingError),

    /// Reading or writing a package file failed.
    Io(IOError),

    /// A payload was readable but semantically malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Json(e) => e.fmt(f),
            Error::Plist(e) => e.fmt(f),
            Error::PngDecode(e) => e.fmt(f),
            Error::PngEncode(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Json(e) => Some(e),
            Error::Plist(e) => Some(e),
            Error::PngDecode(e) => Some(e),
            Error::PngEncode(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::Invalid(_) => None,
        }
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}

impl From<PlistError> for Error {
    fn from(e: PlistError) -> Self {
        Error::Plist(e)
    }
}

impl From<DecodingError> for Error {
    fn from(e: DecodingError) -> Self {
        Error::PngDecode(e)
    }
}

impl From<EncodingError> for Error {
    fn from(e: EncodingError) -> Self {
        Error::PngEncode(e)
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::Io(e)
    }
}
