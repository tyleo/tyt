use crate::{Error, Result};
use openxr::sys::Result as XrResult;

/// `result` as an error when it reports a failure, for raw OpenXR calls.
pub fn check_xr(result: XrResult) -> Result<XrResult> {
    if result.into_raw() < 0 {
        Err(Error::Xr(result))
    } else {
        Ok(result)
    }
}
