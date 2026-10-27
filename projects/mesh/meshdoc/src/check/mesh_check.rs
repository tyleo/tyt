use crate::check::MeshCheckStatus;

/// One check's outcome over a document, whatever format ran it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshCheck {
    /// A short stable identifier, such as `decode`.
    pub name: &'static str,

    /// The outcome.
    pub status: MeshCheckStatus,
}

impl MeshCheck {
    /// A check that ran and found no problems.
    pub fn passed(name: &'static str) -> Self {
        Self {
            name,
            status: MeshCheckStatus::Passed,
        }
    }

    /// A check that ran and found `messages`, one per problem.
    pub fn failed(name: &'static str, messages: Vec<String>) -> Self {
        Self {
            name,
            status: MeshCheckStatus::Failed(messages),
        }
    }
}
