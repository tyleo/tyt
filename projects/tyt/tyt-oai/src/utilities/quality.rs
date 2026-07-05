use clap::ValueEnum;

/// The rendering quality of a generated image.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Quality {
    /// Let the model choose the quality.
    Auto,
    /// Low quality — fastest and cheapest.
    Low,
    /// Medium quality.
    Medium,
    /// High quality — slowest and most detailed.
    High,
}

impl Quality {
    /// Returns the string passed to the `image_generation` tool's `quality`
    /// parameter.
    pub fn as_api_str(self) -> &'static str {
        match self {
            Quality::Auto => "auto",
            Quality::Low => "low",
            Quality::Medium => "medium",
            Quality::High => "high",
        }
    }
}
