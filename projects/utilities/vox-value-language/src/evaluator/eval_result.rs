use crate::EvalFailure;
use std::result::Result as StdResult;

/// An evaluation result, the failure not yet tied to its binding.
pub(crate) type EvalResult<T> = StdResult<T, EvalFailure>;
