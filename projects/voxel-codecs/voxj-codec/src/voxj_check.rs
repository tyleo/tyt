use crate::VoxjCheckStatus;

/// One named validation check and how a [`VoxjFile`](voxj::VoxjFile) fared on
/// it, produced by [`check_voxj_file`](crate::check_voxj_file()). See
/// [`check_voxj_file`](crate::check_voxj_file()) for the full list of checks.
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjCheck {
    /// A short stable identifier for the check, such as `"tight-bounds"`.
    pub name: &'static str,

    /// The check's outcome.
    pub status: VoxjCheckStatus,
}
