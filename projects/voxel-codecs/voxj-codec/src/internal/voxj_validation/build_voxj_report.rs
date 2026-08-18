use crate::{Check, VoxjCheck, VoxjCheckStatus};

/// Every check, in the order [`check_voxj_file`](crate::check_voxj_file())
/// reports them.
const REPORT_ORDER: [Check; 12] = [
    Check::Version,
    Check::Palettes,
    Check::Indices,
    Check::Blocks,
    Check::UniquePositions,
    Check::Bounds,
    Check::SampleMaterials,
    Check::Acyclic,
    Check::Scale,
    Check::Rotation,
    Check::EditState,
    Check::SampleOrder,
];

/// Groups tagged failures into one [`VoxjCheck`] per check, in
/// [`REPORT_ORDER`]. A check with no failures passed; [`Check::SampleOrder`] is
/// always unverifiable.
pub fn build_voxj_report(failures: Vec<(Check, String)>) -> Vec<VoxjCheck> {
    REPORT_ORDER
        .iter()
        .map(|&check| {
            let status = if check == Check::SampleOrder {
                VoxjCheckStatus::Unverifiable
            } else {
                let messages: Vec<String> = failures
                    .iter()
                    .filter(|(c, _)| *c == check)
                    .map(|(_, message)| message.clone())
                    .collect();
                if messages.is_empty() {
                    VoxjCheckStatus::Passed
                } else {
                    VoxjCheckStatus::Failed(messages)
                }
            };
            VoxjCheck {
                name: check.name(),
                status,
            }
        })
        .collect()
}
