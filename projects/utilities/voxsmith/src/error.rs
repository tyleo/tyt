#[cfg(feature = "mesh")]
use crate::operations::mesh::MeshElement;
use meshdoc::Error as MeshError;
use pathspec::Error as PathSpecError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[cfg(feature = "_treegrid")]
use treegrid::TreeGridError;
use voxcore::Error as VoxError;

/// An error from voxsmith.
#[derive(Debug)]
pub enum Error {
    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// A voxcore construction, mutation, or insertion was rejected.
    Vox(VoxError),

    /// A meshdoc construction, mutation, or insertion was rejected.
    Mesh(MeshError),

    /// A mesh record element the run could not mesh.
    #[cfg(feature = "mesh")]
    MeshRecord {
        /// The element the error rose from.
        element: MeshElement,

        /// What went wrong with it.
        reason: String,
    },

    /// An image could not be encoded as PNG.
    #[cfg(feature = "mesh")]
    Png(String),

    /// An image of the mesh document could not be decoded.
    #[cfg(feature = "voxelize")]
    DecodeImage(String),

    /// A report layout rejected an option it does not consume.
    #[cfg(feature = "_treegrid")]
    TreeGrid(TreeGridError),

    /// A hierarchy-path pattern is not a valid gitignore-style glob.
    PathSpec(PathSpecError),
}

impl Error {
    /// Builds an [`Error::Invalid`] from a message.
    pub(crate) fn invalid(message: impl Display) -> Self {
        Error::Invalid(message.to_string())
    }

    /// Builds an [`Error::MeshRecord`] from the element and its reason.
    #[cfg(feature = "mesh")]
    pub(crate) fn mesh_record(element: MeshElement, reason: impl Display) -> Self {
        Error::MeshRecord {
            element,
            reason: reason.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Vox(error) => error.fmt(f),
            Error::Mesh(error) => error.fmt(f),
            #[cfg(feature = "mesh")]
            Error::MeshRecord { element, reason } => write!(f, "{element} {reason}"),
            #[cfg(feature = "mesh")]
            Error::Png(message) => write!(f, "could not encode PNG: {message}"),
            #[cfg(feature = "voxelize")]
            Error::DecodeImage(message) => write!(f, "could not decode image: {message}"),
            #[cfg(feature = "_treegrid")]
            Error::TreeGrid(error) => error.fmt(f),
            Error::PathSpec(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Invalid(_) => None,
            Error::Vox(error) => Some(error),
            Error::Mesh(error) => Some(error),
            #[cfg(feature = "mesh")]
            Error::MeshRecord { .. } => None,
            #[cfg(feature = "mesh")]
            Error::Png(_) => None,
            #[cfg(feature = "voxelize")]
            Error::DecodeImage(_) => None,
            #[cfg(feature = "_treegrid")]
            Error::TreeGrid(error) => Some(error),
            Error::PathSpec(error) => Some(error),
        }
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}

impl From<MeshError> for Error {
    fn from(error: MeshError) -> Self {
        Error::Mesh(error)
    }
}

#[cfg(feature = "_treegrid")]
impl From<TreeGridError> for Error {
    fn from(error: TreeGridError) -> Self {
        Error::TreeGrid(error)
    }
}

impl From<PathSpecError> for Error {
    fn from(error: PathSpecError) -> Self {
        Error::PathSpec(error)
    }
}
