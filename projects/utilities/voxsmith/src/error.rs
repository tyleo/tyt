use goxl_codec::Error as GoxlError;
use mvox_codec::Error as MVoxError;
use qbcl_codec::Error as QbclError;
use serde::{de::Error as DeError, ser::Error as SerError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use vmax_codec::Error as VMaxError;
use voxcore::Error as VoxError;
use voxj_codec::Error as VoxjError;

/// An error from voxsmith: voxel data that is malformed, a state that fails
/// voxcore validation, a Voxel Json document that fails to encode or decode, a
/// MagicaVoxel `.vox` file that fails to decode, a Voxel Max payload that fails
/// to decode or encode, a Goxel `.gox` file that fails to decode, or a Qubicle
/// `.qb` / `.qbt` / `.qbcl` file that fails to decode.
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

    /// Decoding or encoding a Voxel Max payload failed.
    VMax(VMaxError),

    /// Decoding a Goxel `.gox` file failed.
    Goxl(GoxlError),

    /// Decoding a Qubicle `.qb` / `.qbt` / `.qbcl` file failed.
    Qbcl(QbclError),
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
            Error::VMax(error) => error.fmt(f),
            Error::Goxl(error) => error.fmt(f),
            Error::Qbcl(error) => error.fmt(f),
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
            Error::VMax(error) => Some(error),
            Error::Goxl(error) => Some(error),
            Error::Qbcl(error) => Some(error),
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

impl From<VMaxError> for Error {
    fn from(error: VMaxError) -> Self {
        Error::VMax(error)
    }
}

impl From<GoxlError> for Error {
    fn from(error: GoxlError) -> Self {
        Error::Goxl(error)
    }
}

impl From<QbclError> for Error {
    fn from(error: QbclError) -> Self {
        Error::Qbcl(error)
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
