use crate::validation::VoxjCheckStatus;

/// One named validation check and how a [`VoxjFile`](crate::VoxjFile) fared on
/// it, produced by [`check_voxj_file`](crate::validation::check_voxj_file()),
/// which documents the full list of checks.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjCheck {
    /// A short stable identifier for the check, such as `"tight-bounds"`.
    pub name: &'static str,

    /// The check's outcome.
    pub status: VoxjCheckStatus,
}
