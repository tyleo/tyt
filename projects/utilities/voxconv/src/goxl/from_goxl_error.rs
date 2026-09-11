use crate::Error;
use goxl_voxcore::Error as GoxlError;

/// The bridge's error as a voxconv [`Error::Format`].
impl From<GoxlError> for Error {
    fn from(error: GoxlError) -> Self {
        Error::Format(Box::new(error))
    }
}
