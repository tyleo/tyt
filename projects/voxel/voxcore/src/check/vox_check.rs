use crate::check::VoxCheckStatus;

/// One check's outcome over a document, whatever format ran it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoxCheck {
    /// A short stable identifier, such as `decode` or `tight-bounds`.
    pub name: &'static str,

    /// The outcome.
    pub status: VoxCheckStatus,
}

impl VoxCheck {
    /// A check that ran and found no problems.
    pub fn passed(name: &'static str) -> Self {
        Self {
            name,
            status: VoxCheckStatus::Passed,
        }
    }

    /// A check that ran and found `messages`, one per problem.
    pub fn failed(name: &'static str, messages: Vec<String>) -> Self {
        Self {
            name,
            status: VoxCheckStatus::Failed(messages),
        }
    }
}
