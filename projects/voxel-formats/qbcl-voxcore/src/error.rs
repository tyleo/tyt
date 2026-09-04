#[cfg(feature = "codec")]
use qbcl_codec::Error as CodecError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxcore::Error as VoxError;

/// An error from qbcl-voxcore.
#[derive(Debug)]
pub enum Error {
    /// Reading or writing file bytes failed.
    #[cfg(feature = "codec")]
    Codec(CodecError),

    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// A voxcore construction, mutation, or insertion was rejected.
    Vox(VoxError),
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
            #[cfg(feature = "codec")]
            Error::Codec(error) => error.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Vox(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "codec")]
            Error::Codec(error) => Some(error),
            Error::Invalid(_) => None,
            Error::Vox(error) => Some(error),
        }
    }
}

#[cfg(feature = "codec")]
impl From<CodecError> for Error {
    fn from(error: CodecError) -> Self {
        Error::Codec(error)
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}
