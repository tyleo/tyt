use base64::DecodeError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding an object's voxel-position or voxel-sample
/// blocks.
#[derive(Debug)]
pub enum Error {
    /// A base64-encoded position or sample block could not be decoded.
    Base64(DecodeError),

    /// A block was readable but structurally malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Base64(e) => e.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Base64(e) => Some(e),
            Error::Invalid(_) => None,
        }
    }
}

impl From<DecodeError> for Error {
    fn from(e: DecodeError) -> Self {
        Error::Base64(e)
    }
}
