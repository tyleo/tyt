use crate::VoxjFile;
use crate::validation::{Check, Failures};

/// The only Voxel Json document version this codec understands.
const SUPPORTED_VERSION: u32 = 1;

/// The version is one this codec understands.
pub fn check_version(file: &VoxjFile, failures: &mut Failures) {
    if file.version != SUPPORTED_VERSION {
        failures.report(
            Check::Version,
            format!(
                "unrecognized version {}, expected {SUPPORTED_VERSION}",
                file.version
            ),
        );
    }
}
