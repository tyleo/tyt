use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxcore::{Error as VoxError, ext::Error as ExtError};
use voxj::objects::Error as ObjectsError;
#[cfg(feature = "codec")]
use voxj_codec::Error as CodecError;

/// An error from voxj-voxcore.
#[derive(Debug)]
pub enum Error {
    /// Reading or writing document bytes failed.
    #[cfg(feature = "codec")]
    Codec(CodecError),

    /// A slot's ext failed to encode to or decode from its block form.
    Ext(ExtError),

    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// Decoding or encoding an object's position or sample blocks failed.
    Objects(ObjectsError),

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
            Error::Ext(error) => error.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Objects(error) => error.fmt(f),
            Error::Vox(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "codec")]
            Error::Codec(error) => Some(error),
            Error::Ext(error) => Some(error),
            Error::Invalid(_) => None,
            Error::Objects(error) => Some(error),
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

impl From<ExtError> for Error {
    fn from(error: ExtError) -> Self {
        Error::Ext(error)
    }
}

impl From<ObjectsError> for Error {
    fn from(error: ObjectsError) -> Self {
        Error::Objects(error)
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}
