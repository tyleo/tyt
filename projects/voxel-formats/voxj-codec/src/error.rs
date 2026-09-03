use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding a Voxel Json (`.voxj` / `.voxjz`) document.
#[derive(Debug)]
pub enum Error {
    /// The document JSON could not be parsed.
    Json(String),

    /// The document or `.voxjz` archive was readable but structurally
    /// malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Json(message) | Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}
