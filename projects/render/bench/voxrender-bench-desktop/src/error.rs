use png::EncodingError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use voxrender_bench::Error as BenchError;
use wgpu::{CreateSurfaceError, RequestAdapterError, RequestDeviceError};
use winit::error::{EventLoopError, OsError};

/// Why the app failed.
#[derive(Debug)]
pub enum Error {
    /// No GPU adapter was found.
    Adapter(RequestAdapterError),

    /// The adapter would not open a device.
    Device(RequestDeviceError),

    /// The view or its check failed.
    Bench(BenchError),

    /// No bench scene's name starts with the text.
    NoScene(String),

    /// The image's file would not open.
    Write(IOError),

    /// The image would not encode.
    Png(EncodingError),

    /// The window's event loop failed.
    EventLoop(EventLoopError),

    /// The window would not open.
    Window(OsError),

    /// The window gave no surface to draw on.
    Surface(CreateSurfaceError),

    /// The adapter cannot draw to the window's surface.
    NoSurface,

    /// The window's surface gave no image to draw a frame into.
    NoFrame,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Adapter(error) => write!(f, "no GPU adapter: {error}"),
            Error::Device(error) => write!(f, "no GPU device: {error}"),
            Error::Bench(error) => write!(f, "{error}"),
            Error::NoScene(name) => write!(f, "no bench scene's name starts with {name:?}"),
            Error::Write(error) => write!(f, "the image's file would not open: {error}"),
            Error::Png(error) => write!(f, "the image would not encode: {error}"),
            Error::EventLoop(error) => write!(f, "the window's event loop failed: {error}"),
            Error::Window(error) => write!(f, "the window would not open: {error}"),
            Error::Surface(error) => write!(f, "the window gave no surface: {error}"),
            Error::NoSurface => write!(f, "the adapter cannot draw to the window's surface"),
            Error::NoFrame => write!(f, "the window's surface gave no image to draw into"),
        }
    }
}

impl StdError for Error {}
