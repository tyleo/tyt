/// The reusable parts of a `*.meshy.{mesh,texture}.json` task file: enough to
/// poll the task and rewrite the file in place, preserving its `payload.input`.
#[derive(Clone, Debug)]
pub struct TaskFileHead {
    /// The Meshy task id.
    pub task_id: String,

    /// The task kind (`image-to-3d` or `retexture`), selecting the API and
    /// written back unchanged.
    pub task_kind: String,

    /// The verbatim `payload.input` object, as JSON bytes, preserved when the
    /// file is rewritten.
    pub input_json: Vec<u8>,
}
