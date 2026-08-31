use crate::objects::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is an objects [`Error`].
pub type Result<T> = StdResult<T, Error>;
