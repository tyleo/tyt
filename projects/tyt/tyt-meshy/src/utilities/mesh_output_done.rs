use crate::MeshProcessed;

/// A completed task's output: the verbatim Meshy task object plus the local
/// files written from it.
#[derive(Clone, Debug)]
pub struct MeshOutputDone {
    /// The verbatim Meshy task object, stored as `output.raw`.
    pub raw_json: Vec<u8>,

    /// The local files written from the task, stored as `output.processed`.
    pub processed: MeshProcessed,
}
