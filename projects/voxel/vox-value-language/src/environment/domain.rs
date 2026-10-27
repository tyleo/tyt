use std::fmt::{Display, Formatter, Result as FmtResult};

/// What a value has one entry per; the ladder runs bottom to top.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Domain {
    /// A single entry.
    Plain,

    /// One entry per swatch.
    Swatch,

    /// One entry per solid voxel.
    Voxel,

    /// One entry per emitted face.
    Face,

    /// One entry per face corner.
    Corner,
}

impl Domain {
    /// Whether the domain holds one entry per element rather than one entry.
    pub fn is_array(self) -> bool {
        self != Domain::Plain
    }

    /// The domain's name.
    pub fn name(self) -> &'static str {
        match self {
            Domain::Plain => "plain",
            Domain::Swatch => "swatch",
            Domain::Voxel => "voxel",
            Domain::Face => "face",
            Domain::Corner => "corner",
        }
    }
}

impl Display for Domain {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.name())
    }
}
