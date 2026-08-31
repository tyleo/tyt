#[cfg(feature = "gltf")]
use gltf::Error as GltfError;
#[cfg(feature = "goxl")]
use goxl_codec::Error as GoxlError;
#[cfg(feature = "mvox")]
use mvox_codec::Error as MVoxError;
#[cfg(feature = "qbcl")]
use qbcl_codec::Error as QbclError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[cfg(feature = "vmax")]
use vmax_codec::Error as VMaxError;
use voxcore::Error as VoxError;
#[cfg(feature = "voxj")]
use voxj_voxcore::Error as VoxjError;

/// An error from voxsmith.
#[derive(Debug)]
pub enum Error {
    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// A voxcore construction, mutation, or insertion was rejected.
    Vox(VoxError),

    /// Converting a Voxel Json document failed.
    #[cfg(feature = "voxj")]
    Voxj(VoxjError),

    /// Decoding a MagicaVoxel `.vox` file failed.
    #[cfg(feature = "mvox")]
    MVox(MVoxError),

    /// Decoding or encoding a Voxel Max payload failed.
    #[cfg(feature = "vmax")]
    VMax(VMaxError),

    /// Decoding a Goxel `.gox` file failed.
    #[cfg(feature = "goxl")]
    Goxl(GoxlError),

    /// Decoding a Qubicle `.qb` / `.qbt` / `.qbcl` file failed.
    #[cfg(feature = "qbcl")]
    Qbcl(QbclError),

    /// Reading a glTF or GLB mesh failed.
    #[cfg(feature = "gltf")]
    Gltf(GltfError),
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
            #[cfg(feature = "voxj")]
            Error::Voxj(error) => error.fmt(f),
            #[cfg(feature = "mvox")]
            Error::MVox(error) => error.fmt(f),
            #[cfg(feature = "vmax")]
            Error::VMax(error) => error.fmt(f),
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => error.fmt(f),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => error.fmt(f),
            #[cfg(feature = "gltf")]
            Error::Gltf(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Invalid(_) => None,
            Error::Vox(error) => Some(error),
            #[cfg(feature = "voxj")]
            Error::Voxj(error) => Some(error),
            #[cfg(feature = "mvox")]
            Error::MVox(error) => Some(error),
            #[cfg(feature = "vmax")]
            Error::VMax(error) => Some(error),
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => Some(error),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => Some(error),
            #[cfg(feature = "gltf")]
            Error::Gltf(error) => Some(error),
        }
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}

#[cfg(feature = "voxj")]
impl From<VoxjError> for Error {
    fn from(error: VoxjError) -> Self {
        Error::Voxj(error)
    }
}

#[cfg(feature = "mvox")]
impl From<MVoxError> for Error {
    fn from(error: MVoxError) -> Self {
        Error::MVox(error)
    }
}

#[cfg(feature = "vmax")]
impl From<VMaxError> for Error {
    fn from(error: VMaxError) -> Self {
        Error::VMax(error)
    }
}

#[cfg(feature = "goxl")]
impl From<GoxlError> for Error {
    fn from(error: GoxlError) -> Self {
        Error::Goxl(error)
    }
}

#[cfg(feature = "qbcl")]
impl From<QbclError> for Error {
    fn from(error: QbclError) -> Self {
        Error::Qbcl(error)
    }
}

#[cfg(feature = "gltf")]
impl From<GltfError> for Error {
    fn from(error: GltfError) -> Self {
        Error::Gltf(error)
    }
}
