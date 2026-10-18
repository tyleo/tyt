use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender::Error as RenderError;
use wgpu::{BufferAsyncError, MapRangeError, PollError, RequestAdapterError, RequestDeviceError};

/// Why a benchmark failed to run.
#[derive(Debug)]
pub enum Error {
    /// The reference could not set up a scene's rays.
    Render(RenderError),

    /// No GPU adapter was found.
    Adapter(RequestAdapterError),

    /// The adapter would not open a device.
    Device(RequestDeviceError),

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
            Error::Render(error) => write!(f, "the reference failed: {error}"),
            Error::Adapter(error) => write!(f, "no GPU adapter: {error}"),
            Error::Device(error) => write!(f, "no GPU device: {error}"),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a readback buffer would not map: {error}"),
            Error::MapRange(error) => write!(f, "a readback range was unavailable: {error}"),
        }
    }
}

impl StdError for Error {}
