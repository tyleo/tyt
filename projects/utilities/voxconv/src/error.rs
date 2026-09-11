#[cfg(feature = "goxl")]
use goxl_voxcore::Error as GoxlError;
#[cfg(feature = "mvox")]
use mvox_voxcore::Error as MVoxError;
#[cfg(feature = "qbcl")]
use qbcl_voxcore::Error as QbclError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
#[cfg(feature = "vmax")]
use vmax_voxcore::Error as VMaxError;
#[cfg(feature = "voxj")]
use voxj_voxcore::Error as VoxjError;

/// An error from voxconv.
#[derive(Debug)]
pub enum Error {
    /// A document's files do not fit its format: a single-file format given
    /// more or fewer than one file, or a package entry without a UTF-8 path.
    Files(String),

    /// Reading or writing a document's files failed.
    Io(IOError),

    /// An ext failed to encode to or decode from its Voxel Json entries.
    Ext(String),

    /// Converting a Goxel `.gox` file failed.
    #[cfg(feature = "goxl")]
    Goxl(GoxlError),

    /// Converting a MagicaVoxel `.vox` file failed.
    #[cfg(feature = "mvox")]
    MVox(MVoxError),

    /// Converting a Qubicle `.qb` / `.qbt` / `.qbcl` file failed.
    #[cfg(feature = "qbcl")]
    Qbcl(QbclError),

    /// Converting a Voxel Max package failed.
    #[cfg(feature = "vmax")]
    VMax(VMaxError),

    /// Converting a Voxel Json document failed.
    #[cfg(feature = "voxj")]
    Voxj(VoxjError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Files(message) => write!(f, "{message}"),
            Error::Io(error) => error.fmt(f),
            Error::Ext(message) => write!(f, "{message}"),
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => error.fmt(f),
            #[cfg(feature = "mvox")]
            Error::MVox(error) => error.fmt(f),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => error.fmt(f),
            #[cfg(feature = "vmax")]
            Error::VMax(error) => error.fmt(f),
            #[cfg(feature = "voxj")]
            Error::Voxj(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Files(_) => None,
            Error::Io(error) => Some(error),
            Error::Ext(_) => None,
            #[cfg(feature = "goxl")]
            Error::Goxl(error) => Some(error),
            #[cfg(feature = "mvox")]
            Error::MVox(error) => Some(error),
            #[cfg(feature = "qbcl")]
            Error::Qbcl(error) => Some(error),
            #[cfg(feature = "vmax")]
            Error::VMax(error) => Some(error),
            #[cfg(feature = "voxj")]
            Error::Voxj(error) => Some(error),
        }
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error::Io(error)
    }
}

#[cfg(feature = "goxl")]
impl From<GoxlError> for Error {
    fn from(error: GoxlError) -> Self {
        Error::Goxl(error)
    }
}

#[cfg(feature = "mvox")]
impl From<MVoxError> for Error {
    fn from(error: MVoxError) -> Self {
        Error::MVox(error)
    }
}

#[cfg(feature = "qbcl")]
impl From<QbclError> for Error {
    fn from(error: QbclError) -> Self {
        Error::Qbcl(error)
    }
}

#[cfg(feature = "vmax")]
impl From<VMaxError> for Error {
    fn from(error: VMaxError) -> Self {
        Error::VMax(error)
    }
}

#[cfg(feature = "voxj")]
impl From<VoxjError> for Error {
    fn from(error: VoxjError) -> Self {
        Error::Voxj(error)
    }
}
