use crate::Error;
use vmax_voxcore::Error as VMaxError;

/// The bridge's error as a voxconv [`Error::Format`].
impl From<VMaxError> for Error {
    fn from(error: VMaxError) -> Self {
        Error::Format(Box::new(error))
    }
}
