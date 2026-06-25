use clap::ValueEnum;

/// The Meshy `ai_model` used for generation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum Model {
    /// Meshy 5.
    #[value(name = "meshy-5")]
    Meshy5,

    /// Meshy 6.
    #[default]
    #[value(name = "meshy-6")]
    Meshy6,

    /// The latest model (currently Meshy 6).
    Latest,
}

impl Model {
    /// Returns the string sent as the API's `ai_model`.
    pub fn as_api_str(self) -> &'static str {
        match self {
            Model::Meshy5 => "meshy-5",
            Model::Meshy6 => "meshy-6",
            Model::Latest => "latest",
        }
    }

    /// Returns whether this model supports `image_enhancement`, `remove_lighting`,
    /// HD textures, and emission maps, which require Meshy 6.
    pub fn is_meshy6(self) -> bool {
        matches!(self, Model::Meshy6 | Model::Latest)
    }
}
