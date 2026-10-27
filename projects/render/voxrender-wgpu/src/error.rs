use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender::Error as RenderError;
use wgpu::{BufferAsyncError, MapRangeError, PollError};

/// Why a trace failed.
#[derive(Debug)]
pub enum Error {
    /// The scene's rays could not be set up.
    Render(RenderError),

    /// Waiting on the GPU failed.
    Poll(PollError),

    /// A readback buffer would not map.
    Map(BufferAsyncError),

    /// A mapped readback range was not available.
    MapRange(MapRangeError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Render(error) => write!(f, "the scene's rays failed: {error}"),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a readback buffer would not map: {error}"),
            Error::MapRange(error) => write!(f, "a readback range was unavailable: {error}"),
        }
    }
}

impl StdError for Error {}
