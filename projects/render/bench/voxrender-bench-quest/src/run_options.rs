use crate::{Error, Result};
use std::process::Command;

/// The system property that turns on the profile.
const PROFILE: &str = "debug.voxrenderbench.profile";

/// The system property that turns off B1.
const B1: &str = "debug.voxrenderbench.b1";

/// The system property that holds the text of the kernels B2 times.
const KERNELS: &str = "debug.voxrenderbench.kernels";

/// What the app runs, from the system properties the README's steps set.
pub struct RunOptions {
    /// Whether the profile runs in place of B1 and B2's timer.
    pub profile: bool,

    /// Whether B1 runs before the session opens.
    pub b1: bool,

    /// The text a kernel's label must hold for B2 to time it.
    pub kernels: Option<String>,
}

impl RunOptions {
    /// Reads the properties. Errors unless each flag is unset, `true`, or
    /// `false`.
    pub fn read() -> Result<Self> {
        let kernels = getprop(KERNELS)?;

        Ok(RunOptions {
            profile: flag(PROFILE, false)?,
            b1: flag(B1, true)?,
            kernels: (!kernels.is_empty()).then_some(kernels),
        })
    }
}

/// The value of the system property `name`, or an empty string when the
/// property is unset.
fn getprop(name: &str) -> Result<String> {
    let output = Command::new("getprop")
        .arg(name)
        .output()
        .map_err(Error::Getprop)?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// The flag the system property `name` holds, or `unset` when the property
/// is unset.
fn flag(name: &'static str, unset: bool) -> Result<bool> {
    match getprop(name)?.as_str() {
        "" => Ok(unset),
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(Error::FlagProperty {
            name,
            value: value.to_owned(),
        }),
    }
}
