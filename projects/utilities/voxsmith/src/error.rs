#[cfg(feature = "gltf")]
use gltf::Error as GltfError;
#[cfg(feature = "goxl")]
use goxl_codec::Error as GoxlError;
#[cfg(feature = "mvox")]
use mvox_codec::Error as MVoxError;
#[cfg(feature = "select")]
use pathspec::Error as PathSpecError;
#[cfg(feature = "qbcl")]
use qbcl_codec::Error as QbclError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[cfg(feature = "report")]
use treegrid::TreeGridError;
use voxcore::Error as VoxError;
#[cfg(feature = "voxj")]
use voxj_codec::Error as VoxjCodecError;
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

    /// Decoding a Goxel `.gox` file failed.
    #[cfg(feature = "goxl")]
    Goxl(GoxlError),

    /// Decoding a Qubicle `.qb` / `.qbt` / `.qbcl` file failed.
    #[cfg(feature = "qbcl")]
    Qbcl(QbclError),

    /// Reading a glTF or GLB mesh failed.
    #[cfg(feature = "gltf")]
    Gltf(GltfError),

    /// A report layout rejected an option it does not consume.
    #[cfg(feature = "report")]
    TreeGrid(TreeGridError),

    /// A hierarchy-path pattern is not a valid gitignore-style glob.
    #[cfg(feature = "select")]
    PathSpec(PathSpecError),
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
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => error.fmt(f),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => error.fmt(f),
            #[cfg(feature = "gltf")]
            Error::Gltf(error) => error.fmt(f),
            #[cfg(feature = "report")]
            Error::TreeGrid(error) => error.fmt(f),
            #[cfg(feature = "select")]
            Error::PathSpec(error) => error.fmt(f),
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
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => Some(error),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => Some(error),
            #[cfg(feature = "gltf")]
            Error::Gltf(error) => Some(error),
            #[cfg(feature = "report")]
            Error::TreeGrid(error) => Some(error),
            #[cfg(feature = "select")]
            Error::PathSpec(error) => Some(error),
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

/// A document byte codec failure is the codec case of the Voxel Json error.
#[cfg(feature = "voxj")]
impl From<VoxjCodecError> for Error {
    fn from(error: VoxjCodecError) -> Self {
        Error::Voxj(VoxjError::from(error))
    }
}

#[cfg(feature = "mvox")]
impl From<MVoxError> for Error {
    fn from(error: MVoxError) -> Self {
        Error::MVox(error)
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

#[cfg(feature = "report")]
impl From<TreeGridError> for Error {
    fn from(error: TreeGridError) -> Self {
        Error::TreeGrid(error)
    }
}

#[cfg(feature = "select")]
impl From<PathSpecError> for Error {
    fn from(error: PathSpecError) -> Self {
        Error::PathSpec(error)
    }
}
