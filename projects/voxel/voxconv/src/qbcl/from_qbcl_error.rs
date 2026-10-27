use crate::Error;
use qbcl_voxcore::Error as QbclError;

/// The bridge's error as a voxconv [`Error::Format`].
impl From<QbclError> for Error {
    fn from(error: QbclError) -> Self {
        Error::Format(Box::new(error))
    }
}
