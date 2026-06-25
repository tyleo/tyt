use crate::Error;

/// An [`Error::Invalid`] carrying `message`, for well-framed but semantically
/// malformed input.
pub fn invalid(message: String) -> Error {
    Error::Invalid(message)
}
