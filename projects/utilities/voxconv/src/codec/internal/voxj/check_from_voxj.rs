use voxcore::check::{VoxCheck, VoxCheckStatus};
use voxj::validation::{VoxjCheck, VoxjCheckStatus};

/// A Voxel Json spec check in voxcore's form.
pub fn check_from_voxj(check: VoxjCheck) -> VoxCheck {
    VoxCheck {
        name: check.name,
        status: match check.status {
            VoxjCheckStatus::Passed => VoxCheckStatus::Passed,
            VoxjCheckStatus::Failed(messages) => VoxCheckStatus::Failed(messages),
            VoxjCheckStatus::Unverifiable => VoxCheckStatus::Unverifiable,
        },
    }
}
