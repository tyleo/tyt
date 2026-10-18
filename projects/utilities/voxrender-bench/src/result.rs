use crate::Error;
use std::result::Result as StdResult;

/// A benchmark's result.
pub type Result<T> = StdResult<T, Error>;
