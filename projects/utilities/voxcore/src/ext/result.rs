use crate::ext::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is an ext-protocol [`Error`].
pub type Result<T> = StdResult<T, Error>;
