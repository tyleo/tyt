use crate::check::VoxCheck;

/// How many of `checks` failed.
pub fn failed_check_count(checks: &[VoxCheck]) -> usize {
    checks
        .iter()
        .filter(|check| check.status.is_failed())
        .count()
}

#[cfg(test)]
mod tests {
    use crate::check::{VoxCheck, VoxCheckStatus, failed_check_count};

    #[test]
    fn counts_only_failures() {
        let checks = [
            VoxCheck::passed("a"),
            VoxCheck::failed("b", vec!["broken".to_owned()]),
            VoxCheck {
                name: "c",
                status: VoxCheckStatus::Unverifiable,
            },
        ];

        assert_eq!(failed_check_count(&checks), 1);

        assert_eq!(failed_check_count(&[]), 0);
    }
}
