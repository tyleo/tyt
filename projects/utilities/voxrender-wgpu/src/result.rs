use crate::Error;
use std::result::Result as StdResult;

/// A trace's result.
pub type Result<T> = StdResult<T, Error>;
