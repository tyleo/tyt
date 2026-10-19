use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender_bench::Error as BenchError;
use wgpu::{RequestAdapterError, RequestDeviceError};

/// Why the app failed to run a benchmark.
#[derive(Debug)]
pub enum Error {
    /// No GPU adapter was found.
    Adapter(RequestAdapterError),

    /// The adapter would not open a device.
    Device(RequestDeviceError),

    /// The benchmark failed.
    Bench(BenchError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Adapter(error) => write!(f, "no GPU adapter: {error}"),
            Error::Device(error) => write!(f, "no GPU device: {error}"),
            Error::Bench(error) => write!(f, "{error}"),
        }
    }
}

impl StdError for Error {}
