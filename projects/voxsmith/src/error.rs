use mvox_codec::Error as MVoxError;
use serde::{de::Error as DeError, ser::Error as SerError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxcore::Error as VoxError;
use voxj_codec::Error as VoxjError;

/// An error from voxsmith: voxel data that is malformed, a state that fails
/// voxcore validation, a Voxel Json document that fails to encode or decode, or
/// a MagicaVoxel `.vox` file that fails to decode.
#[derive(Debug)]
pub enum Error {
    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// The assembled state failed voxcore validation.
    Vox(VoxError),

    /// Encoding or decoding a Voxel Json document failed.
    Voxj(VoxjError),

    /// Decoding a MagicaVoxel `.vox` file failed.
    MVox(MVoxError),
}

impl Error {
    /// Builds an [`Error::Invalid`] from a message.
    pub(crate) fn invalid(message: impl Display) -> Self {
        Error::Invalid(message.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Vox(error) => error.fmt(f),
            Error::Voxj(error) => error.fmt(f),
            Error::MVox(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Invalid(_) => None,
            Error::Vox(error) => Some(error),
            Error::Voxj(error) => Some(error),
            Error::MVox(error) => Some(error),
        }
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}

impl From<VoxjError> for Error {
    fn from(error: VoxjError) -> Self {
        Error::Voxj(error)
    }
}

impl From<MVoxError> for Error {
    fn from(error: MVoxError) -> Self {
        Error::MVox(error)
    }
}

impl SerError for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error::invalid(message)
    }
}

impl DeError for Error {
    fn custom<T: Display>(message: T) -> Self {
        Error::invalid(message)
    }
}
