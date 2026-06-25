use crate::Error;
use std::result::Result as StdResult;

/// A result from a cubemap operation.
pub type Result<T> = StdResult<T, Error>;
