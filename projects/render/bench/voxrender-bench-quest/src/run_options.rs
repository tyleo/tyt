use crate::{Error, Result};
use std::process::Command;

/// The system property the README's profile steps set to turn on the
/// profile.
const PROFILE_PROPERTY: &str = "debug.voxrenderbench.profile";

/// Whether the profile runs in place of B1 and B2's timer. Errors unless
/// the property is unset, `true`, or `false`.
pub fn profile_mode() -> Result<bool> {
    let output = Command::new("getprop")
        .arg(PROFILE_PROPERTY)
        .output()
        .map_err(Error::Getprop)?;

    let value = String::from_utf8_lossy(&output.stdout);

    match value.trim() {
        "" | "false" => Ok(false),
        "true" => Ok(true),
        value => Err(Error::ProfileProperty(value.to_owned())),
    }
}
