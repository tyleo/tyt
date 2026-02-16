use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

#[derive(Debug)]
pub enum Error {
    Blender {
        exit_code: Option<i32>,
        stderr: String,
    },
    IO(IOError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Blender { stderr, .. } => write!(f, "blender: {stderr}"),
            Error::IO(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Blender { .. } => None,
            Error::IO(e) => Some(e),
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
