use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use voxrender::Error as RenderError;
use voxrender_wgpu::Error as WgpuError;

/// Why a benchmark failed to run.
#[derive(Debug)]
pub enum Error {
    /// The reference could not set up a scene's rays.
    Render(RenderError),

    /// The GPU tier's trace failed.
    Trace(WgpuError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Render(error) => write!(f, "the reference failed: {error}"),
            Error::Trace(error) => write!(f, "the GPU trace failed: {error}"),
        }
    }
}

impl StdError for Error {}
