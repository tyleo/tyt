use crate::Error;

/// An [`Error::Invalid`] error carrying `message`, for input that is well-framed
/// but semantically malformed.
pub fn invalid(message: String) -> Error {
    Error::Invalid(message)
}
