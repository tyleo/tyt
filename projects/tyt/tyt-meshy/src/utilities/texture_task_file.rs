use crate::{MeshOutput, TextureInput};

/// The full content of a `*.meshy.texture.json` task file.
#[derive(Clone, Debug)]
pub struct TextureTaskFile {
    /// The Meshy task id.
    pub task_id: String,

    /// The fields sent in the create request.
    pub input: TextureInput,

    /// The task output, pending or complete.
    pub output: MeshOutput,
}
