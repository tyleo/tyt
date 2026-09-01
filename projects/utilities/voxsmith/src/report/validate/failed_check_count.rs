use crate::{VoxjCheck, VoxjCheckStatus};

/// How many of `checks` failed.
pub fn failed_check_count(checks: &[VoxjCheck]) -> usize {
    checks
        .iter()
        .filter(|check| matches!(check.status, VoxjCheckStatus::Failed(_)))
        .count()
}
