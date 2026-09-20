use crate::CheckFailure;
use std::result::Result as StdResult;

/// A checking result, the failure not yet tied to its binding.
pub(crate) type CheckResult<T> = StdResult<T, CheckFailure>;
