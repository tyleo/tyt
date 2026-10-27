use crate::Error;

/// An [`Error::Invalid`] carrying `message`.
pub fn invalid(message: String) -> Error {
    Error::Invalid(message)
}
