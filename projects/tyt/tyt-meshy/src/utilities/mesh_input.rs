/// The `payload.input` of a `*.meshy.mesh.json` file: the fields sent in the
/// create request, using Meshy's API field names.
///
/// The field order matches the order written to the file. `image` (and
/// `texture_image`) hold the local path relative to the task file, whereas the
/// create request sends them inline as the API's base64 `image_url`
/// (`texture_image_url`). Fields that do not apply to a run are `None` and are
/// omitted entirely.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "impl", derive(serde::Serialize, serde::Deserialize))]
pub struct MeshInput {
    /// The input image's local path, relative to the task file.
    pub image: String,

    /// The Meshy `model_type` (`standard` or `lowpoly`).
    pub model_type: String,

    /// The Meshy `ai_model`. Omitted in `lowpoly` mode.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub ai_model: Option<String>,

    /// Whether to texture the model.
    pub should_texture: bool,

    /// Whether to generate PBR maps. Present only when `should_texture` is set.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub enable_pbr: Option<bool>,

    /// Whether to generate an HD base color texture. Present only when
    /// `should_texture` is set.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub hd_texture: Option<bool>,

    /// A text prompt guiding texturing. Present only when given.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub texture_prompt: Option<String>,

    /// A texture guidance image's local path, relative to the task file. Present
    /// only when given; sent to the API as the base64 `texture_image_url`.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub texture_image: Option<String>,

    /// Whether to run the remesh phase. Omitted in `lowpoly` mode.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub should_remesh: Option<bool>,

    /// The remesh topology. Present only when remeshing.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub topology: Option<String>,

    /// The target polygon count. Present only when remeshing without a fixed
    /// decimation mode.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub target_polycount: Option<u32>,

    /// The adaptive decimation level. Present only when given.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub decimation_mode: Option<u8>,

    /// Whether to also save the pre-remesh GLB. Present only when remeshing.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub save_pre_remeshed_model: Option<bool>,

    /// Whether to optimize the input image. Present only on Meshy 6.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub image_enhancement: Option<bool>,

    /// Whether to remove highlights and shadows from the base color texture.
    /// Present only on Meshy 6.
    #[cfg_attr(feature = "impl", serde(skip_serializing_if = "Option::is_none"))]
    pub remove_lighting: Option<bool>,

    /// The 3D file formats to generate.
    pub target_formats: Vec<String>,

    /// The Meshy `pose_mode` (always empty).
    pub pose_mode: String,

    /// Whether to moderate the input (always false).
    pub moderation: bool,

    /// Whether to auto-size the model (always false).
    pub auto_size: bool,

    /// Whether to render the four cardinal-view thumbnails (front, right, back,
    /// left) in addition to the default one.
    pub multi_view_thumbnails: bool,
}
