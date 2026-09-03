use crate::snapshots::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is a snapshots [`Error`].
pub type Result<T> = StdResult<T, Error>;
