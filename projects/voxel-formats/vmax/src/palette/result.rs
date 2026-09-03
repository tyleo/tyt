use crate::palette::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a palette [`Error`].
pub type Result<T> = StdResult<T, Error>;
