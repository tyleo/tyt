use crate::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a pathspec [`Error`].
pub type Result<T> = StdResult<T, Error>;
