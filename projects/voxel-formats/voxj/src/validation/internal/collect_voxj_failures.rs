use crate::VoxjFile;
use crate::validation::{
    Check, Failures, check_acyclic, check_edit_state, check_geometry, check_indices,
    check_palettes, check_transforms, check_version,
};

/// Runs every check over `file`, returning the failures it found in discovery
/// order. With `fail_fast`, scanning stops at the first failure; otherwise it
/// continues so a caller can report them all.
///
/// The checks below the version are independent except where one decodes data
/// an earlier check guards: an object's blocks decode only when its palette
/// refs resolve, and the acyclicity walk treats an out-of-range child edge,
/// already reported by [`Check::Indices`], as absent. Such checks skip the work
/// rather than double-report.
pub fn collect_voxj_failures(file: &VoxjFile, fail_fast: bool) -> Vec<(Check, String)> {
    let mut failures = Failures::new(fail_fast);

    check_version(file, &mut failures);
    if failures.go() {
        check_palettes(&file.main, &mut failures);
    }
    if failures.go() {
        check_indices(&file.main, &mut failures);
    }
    if failures.go() {
        check_geometry(&file.main, &mut failures);
    }
    if failures.go() {
        check_acyclic(&file.main, &mut failures);
    }
    if failures.go() {
        check_transforms(&file.main, &mut failures);
    }
    if failures.go() {
        check_edit_state(&file.main, &mut failures);
    }

    failures.into_items()
}
