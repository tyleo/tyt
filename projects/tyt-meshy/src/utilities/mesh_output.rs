use crate::MeshOutputDone;

/// The `payload.output` of a task file: `"pending"` until the task completes,
/// then the completed output.
#[derive(Clone, Debug)]
pub enum MeshOutput {
    /// The task has not completed; serialized as the string `"pending"`.
    Pending,

    /// The task completed and its result files were written.
    Done(MeshOutputDone),
}
