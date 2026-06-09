/// The local files written from a completed task, each path relative to the task
/// file. Each list is ordered so the task file is written stably.
#[derive(Clone, Debug, Default)]
pub struct MeshProcessed {
    /// Model files as `(format, path)` pairs.
    pub model_files: Vec<(String, String)>,

    /// Texture maps as `(map, path)` pairs (`albedo`, `metallic`, `normal`,
    /// `roughness`, `emission`).
    pub texture_files: Vec<(String, String)>,

    /// Thumbnails as `(name, path)` pairs (`default`).
    pub thumbnail_files: Vec<(String, String)>,
}
