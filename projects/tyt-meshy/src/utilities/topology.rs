use clap::ValueEnum;

/// The topology of a remeshed model.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum Topology {
    /// Quad-dominant mesh.
    Quad,

    /// Decimated triangle mesh.
    #[default]
    Triangle,
}

impl Topology {
    /// Returns the string sent as the API's `topology`.
    pub fn as_api_str(self) -> &'static str {
        match self {
            Topology::Quad => "quad",
            Topology::Triangle => "triangle",
        }
    }
}
