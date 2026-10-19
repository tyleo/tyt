use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender::Error as RenderError;
use voxrender_wgpu::Error as WgpuError;
use wgpu::{BufferAsyncError, MapRangeError, PollError};

/// Why a benchmark failed to run.
#[derive(Debug)]
pub enum Error {
    /// The reference could not set up a scene's rays.
    Render(RenderError),

    /// The GPU tier's trace failed.
    Trace(WgpuError),

    /// The device lacks timestamp queries, which a timed benchmark needs.
    NoTimestampQuery,

    /// Waiting on the GPU failed.
    Poll(PollError),

    /// A timestamp readback would not map.
    Map(BufferAsyncError),

    /// A mapped timestamp range was not available.
    MapRange(MapRangeError),

    /// A pass's end timestamp came before its start.
    TimestampOrder,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Render(error) => write!(f, "the reference failed: {error}"),
            Error::Trace(error) => write!(f, "the GPU trace failed: {error}"),
            Error::NoTimestampQuery => write!(f, "the device lacks timestamp queries"),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a timestamp readback would not map: {error}"),
            Error::MapRange(error) => write!(f, "a timestamp range was unavailable: {error}"),
            Error::TimestampOrder => write!(f, "a pass ended before it began"),
        }
    }
}

impl StdError for Error {}
