use crate::Error;
use voxj_voxcore::Error as VoxjError;

/// The bridge's error as a voxconv [`Error::Format`].
impl From<VoxjError> for Error {
    fn from(error: VoxjError) -> Self {
        Error::Format(Box::new(error))
    }
}
