use crate::Error;
use std::result::Result as StdResult;

/// A [`Result`](StdResult) whose error is this crate's [`Error`].
pub type Result<T> = StdResult<T, Error>;
