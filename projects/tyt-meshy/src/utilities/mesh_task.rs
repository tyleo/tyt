/// A parsed Meshy image-to-3D task, as returned by the retrieve endpoint.
#[derive(Clone, Debug)]
pub struct MeshTask {
    /// The task status (`PENDING`, `IN_PROGRESS`, `SUCCEEDED`, `FAILED`, or
    /// `CANCELED`).
    pub status: String,

    /// Completion progress, from 0 to 100.
    pub progress: u8,

    /// Generated model URLs as `(format, url)` pairs (e.g. `("usdz", ...)`),
    /// including `pre_remeshed_glb` when present.
    pub model_urls: Vec<(String, String)>,

    /// Texture map URLs as `(map, url)` pairs using Meshy's names (`base_color`,
    /// `metallic`, `normal`, `roughness`, `emission`), taken from the first
    /// texture set.
    pub texture_urls: Vec<(String, String)>,

    /// The preview thumbnail URL, if any.
    pub thumbnail_url: Option<String>,

    /// The `task_error.message`, when the task carries a non-empty one.
    pub error_message: Option<String>,

    /// The verbatim response body, stored as `output.raw` in the task file.
    pub raw_json: Vec<u8>,
}
