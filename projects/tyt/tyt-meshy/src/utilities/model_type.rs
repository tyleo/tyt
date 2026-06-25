use clap::ValueEnum;

/// The Meshy `model_type`, selecting the generation pipeline.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum ModelType {
    /// Regular high-detail mesh generation.
    #[default]
    Standard,

    /// Low-poly mesh optimized for cleaner polygons.
    Lowpoly,
}

impl ModelType {
    /// Returns the string sent as the API's `model_type`.
    pub fn as_api_str(self) -> &'static str {
        match self {
            ModelType::Standard => "standard",
            ModelType::Lowpoly => "lowpoly",
        }
    }
}
