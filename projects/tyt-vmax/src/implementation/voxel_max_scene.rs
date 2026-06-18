use serde::{Deserialize, Serialize};
use tyt_injection::serde_json::{Map, Value};

/// Scene-level Voxel Max state with no native voxj home: the `scene.json`
/// top-level minus `objects`/`groups`. The codable version is typed; the camera
/// and the renderer/post-grading/UI keys (`cam`, `sat`, `bloom*`, `ao`,
/// `background`, …) are preserved verbatim so they round-trip exactly without
/// this converter having to interpret them.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VoxelMaxScene {
    /// Codable version (`v`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<i64>,
    /// Camera/light rig (`cam`), preserved verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cam: Option<Value>,
    /// All remaining scene-level keys (renderer/post-grading and UI state),
    /// preserved verbatim.
    #[serde(flatten)]
    pub settings: Map<String, Value>,
}
