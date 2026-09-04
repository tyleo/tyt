use crate::Error;
use std::result::Result as StdResult;

/// A `Result` whose error is an mvox-voxcore [`Error`].
pub type Result<T> = StdResult<T, Error>;
