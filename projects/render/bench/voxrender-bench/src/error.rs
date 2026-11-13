use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    path::PathBuf,
};
use voxrender::Error as RenderError;
use wgpu::{BufferAsyncError, MapRangeError, PollError};

/// Why a view, a gallery, or a check failed.
#[derive(Debug)]
pub enum Error {
    /// The reference could not set up a scene's rays.
    Render(RenderError),

    /// No bench scene's name starts with the text.
    NoScene(String),

    /// A model's document gave no scene.
    Model { path: PathBuf, error: String },

    /// A directory gave no gallery of models.
    Models { directory: PathBuf, error: String },

    /// A view cannot look from `eye` at `look_at`: the points coincide, or
    /// one lies straight above the other.
    Look { eye: [f64; 3], look_at: [f64; 3] },

    /// The scene has no view to draw.
    NoView,

    /// The scene has no directional light.
    NoSun,

    /// Waiting on the GPU failed.
    Poll(PollError),

    /// A readback would not map.
    Map(BufferAsyncError),

    /// A mapped readback range was not available.
    MapRange(MapRangeError),

    /// A tile holds more materials than its cells' entries index.
    TileMaterials { materials: usize },

    /// A material that passes light has an id over what a shadow ray's
    /// tile holds for a cell.
    PassingMaterial { id: u32 },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Render(error) => write!(f, "the reference failed: {error}"),
            Error::NoScene(name) => write!(f, "no bench scene's name starts with {name:?}"),
            Error::Model { path, error } => {
                write!(f, "the model at {} gave no scene: {error}", path.display())
            }
            Error::Models { directory, error } => write!(
                f,
                "the models under {} gave no gallery: {error}",
                directory.display()
            ),
            Error::Look { eye, look_at } => write!(
                f,
                "a view cannot look from {eye:?} at {look_at:?}: the points coincide or one lies \
                 straight above the other"
            ),
            Error::NoView => write!(f, "the scene has no view"),
            Error::NoSun => write!(f, "the scene has no directional light"),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a readback would not map: {error}"),
            Error::MapRange(error) => write!(f, "a readback range was unavailable: {error}"),
            Error::TileMaterials { materials } => {
                write!(f, "a tile holds {materials} materials, over 256")
            }
            Error::PassingMaterial { id } => {
                write!(f, "material {id} passes light and its id is over 65534")
            }
        }
    }
}

impl StdError for Error {}
