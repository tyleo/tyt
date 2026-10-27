/// A check's outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeshCheckStatus {
    /// Ran and found no problems.
    Passed,

    /// Ran and found problems, one message each, in discovery order.
    Failed(Vec<String>),

    /// An authoring invariant no document can witness, so neither passed
    /// nor failed.
    Unverifiable,
}

impl MeshCheckStatus {
    /// Whether the check found problems.
    pub fn is_failed(&self) -> bool {
        matches!(self, MeshCheckStatus::Failed(_))
    }
}
