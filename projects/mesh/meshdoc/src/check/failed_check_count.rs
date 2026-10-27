use crate::check::MeshCheck;

/// How many of `checks` failed.
pub fn failed_check_count(checks: &[MeshCheck]) -> usize {
    checks
        .iter()
        .filter(|check| check.status.is_failed())
        .count()
}

#[cfg(test)]
mod tests {
    use crate::check::{MeshCheck, MeshCheckStatus, failed_check_count};

    #[test]
    fn counts_only_failures() {
        let checks = [
            MeshCheck::passed("a"),
            MeshCheck::failed("b", vec!["broken".to_owned()]),
            MeshCheck {
                name: "c",
                status: MeshCheckStatus::Unverifiable,
            },
        ];

        assert_eq!(failed_check_count(&checks), 1);

        assert_eq!(failed_check_count(&[]), 0);
    }
}
