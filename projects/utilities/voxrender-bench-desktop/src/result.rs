use crate::Error;
use std::result::Result as StdResult;

/// The app's result.
pub type Result<T> = StdResult<T, Error>;
