/// The `payload.input` of a `*.meshy.texture.json` file: the fields sent in the
/// retexture create request, using Meshy's API field names.
///
/// The field order matches the order written to the file. `image_style_url`
/// holds the local path relative to the task file, whereas the create request
/// sends it inline as a base64 data URI. Fields that do not apply to a run are
/// `None` and are omitted entirely.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "impl", derive(serde::Serialize, serde::Deserialize))]
pub struct TextureInput {
    /// The source task whose mesh is retextured (its `taskId`).
    pub input_task_id: String,

    /// A text prompt describing the texture style. Present for a text style.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub text_style_prompt: Option<String>,

    /// A style image's local path, relative to the task file. Present for an
    /// image style; sent to the API as a base64 `image_style_url`.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub image_style_url: Option<String>,

    /// The Meshy `ai_model`.
    pub ai_model: String,

    /// Whether to generate PBR maps.
    pub enable_pbr: bool,

    /// Whether to generate an HD base color texture.
    pub hd_texture: bool,

    /// Whether to reuse the source model's original UVs.
    pub enable_original_uv: bool,

    /// Whether to remove highlights and shadows from the base color texture.
    /// Present only on Meshy 6.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub remove_lighting: Option<bool>,

    /// The 3D file formats to generate.
    pub target_formats: Vec<String>,

    /// Whether to moderate the input (always false).
    pub moderation: bool,
}
