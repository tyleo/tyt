use crate::Error;
use mvox_voxcore::Error as MVoxError;

/// The bridge's error as a voxconv [`Error::Format`].
impl From<MVoxError> for Error {
    fn from(error: MVoxError) -> Self {
        Error::Format(Box::new(error))
    }
}
