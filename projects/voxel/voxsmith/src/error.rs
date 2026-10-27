#[cfg(feature = "object")]
use crate::operations::object::MeshElement;
#[cfg(feature = "render")]
use crate::operations::object::RenderElement;
#[cfg(feature = "palette")]
use crate::operations::palette::PaletteEditElement;
#[cfg(feature = "palette")]
use branded_id::U32Id;
use meshdoc::Error as MeshError;
use pathspec::Error as PathSpecError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[cfg(feature = "_treegrid")]
use treegrid::TreeGridError;
#[cfg(feature = "palette")]
use voxcore::BVoxPalette;
use voxcore::Error as VoxError;
#[cfg(feature = "render")]
use voxrender::Error as RenderError;

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
    #[cfg(feature = "object")]
    MeshRecord {
        /// The element the error rose from.
        element: MeshElement,

        /// What went wrong with it.
        reason: String,
    },

    /// An image could not be encoded as PNG.
    #[cfg(feature = "object")]
    Png(String),

    /// A palette edit element the run could not apply.
    #[cfg(feature = "palette")]
    PaletteEdit {
        /// The element the error rose from.
        element: PaletteEditElement,

        /// What went wrong with it.
        reason: String,
    },

    /// An error one palette of a run over several palettes raised.
    #[cfg(feature = "palette")]
    InPalette {
        /// The palette the error rose from.
        palette_id: U32Id<BVoxPalette>,

        /// The error.
        error: Box<Error>,
    },

    /// A palette property the mesher reads for a surface's opacity is bound
    /// to a value pool of the wrong kind.
    #[cfg(feature = "object")]
    MaterialPropertyKind {
        /// The property's name.
        property: String,
    },

    /// A render record element the run could not render.
    #[cfg(feature = "render")]
    RenderRecord {
        /// The element the error rose from.
        element: RenderElement,

        /// What went wrong with it.
        reason: String,
    },

    /// A voxrender scene construction, mutation, or render was rejected.
    #[cfg(feature = "render")]
    Render(RenderError),

    /// An image of the mesh document could not be decoded.
    #[cfg(feature = "mesh_doc")]
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
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Invalid(message) => write!(f, "{message}"),

            Error::Vox(error) => error.fmt(f),

            Error::Mesh(error) => error.fmt(f),

            #[cfg(feature = "object")]
            Error::MeshRecord { element, reason } => write!(f, "{element} {reason}"),

            #[cfg(feature = "object")]
            Error::Png(message) => write!(f, "could not encode PNG: {message}"),

            #[cfg(feature = "palette")]
            Error::PaletteEdit { element, reason } => write!(f, "{element} {reason}"),

            #[cfg(feature = "palette")]
            Error::InPalette { palette_id, error } => write!(f, "palette {palette_id}: {error}"),

            #[cfg(feature = "object")]
            Error::MaterialPropertyKind { property } => write!(
                f,
                "property {property} is not the kind the mesher reads for opacity"
            ),

            #[cfg(feature = "render")]
            Error::RenderRecord { element, reason } => write!(f, "{element} {reason}"),

            #[cfg(feature = "render")]
            Error::Render(error) => error.fmt(f),

            #[cfg(feature = "mesh_doc")]
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

            #[cfg(feature = "object")]
            Error::MeshRecord { .. } => None,

            #[cfg(feature = "object")]
            Error::Png(_) => None,

            #[cfg(feature = "palette")]
            Error::PaletteEdit { .. } => None,

            #[cfg(feature = "palette")]
            Error::InPalette { error, .. } => Some(error.as_ref()),

            #[cfg(feature = "object")]
            Error::MaterialPropertyKind { .. } => None,

            #[cfg(feature = "render")]
            Error::RenderRecord { .. } => None,

            #[cfg(feature = "render")]
            Error::Render(error) => Some(error),

            #[cfg(feature = "mesh_doc")]
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

#[cfg(feature = "render")]
impl From<RenderError> for Error {
    fn from(error: RenderError) -> Self {
        Error::Render(error)
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
