use crate::validation::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a validation [`Error`].
pub type Result<T> = StdResult<T, Error>;
