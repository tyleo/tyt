use crate::{MeshInput, MeshOutput};

/// The full content of a `*.meshy.mesh.json` task file.
#[derive(Clone, Debug)]
pub struct MeshTaskFile {
    /// The Meshy task id.
    pub task_id: String,

    /// The fields sent in the create request.
    pub input: MeshInput,

    /// The task output, pending or complete.
    pub output: MeshOutput,
}
