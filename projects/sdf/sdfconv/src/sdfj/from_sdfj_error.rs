use crate::Error;
use sdfj_sdfcore::Error as SdfjError;

/// The bridge's error as an sdfconv [`Error::Format`].
impl From<SdfjError> for Error {
    fn from(error: SdfjError) -> Self {
        Error::Format(Box::new(error))
    }
}
