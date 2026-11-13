use crate::Error;
use std::result::Result as StdResult;

/// The crate's result.
pub type Result<T> = StdResult<T, Error>;
