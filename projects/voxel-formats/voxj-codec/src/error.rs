use serde_json::Error as JsonError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding a Voxel Json (`.voxj` / `.voxjz`) document.
#[derive(Debug)]
pub enum Error {
    /// The document JSON could not be deserialized or serialized.
    Json(JsonError),

    /// The document or `.voxjz` archive was readable but structurally
    /// malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Json(e) => e.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Json(e) => Some(e),
            Error::Invalid(_) => None,
        }
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}
