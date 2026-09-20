use std::fmt::{Display, Formatter, Result as FmtResult};
use vox_value_language::Domain;

/// An array domain; the ladder runs bottom to top.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArrayDomain {
    /// One entry per swatch.
    Swatch,

    /// One entry per solid voxel.
    Voxel,

    /// One entry per emitted face.
    Face,

    /// One entry per face corner.
    Corner,
}

impl ArrayDomain {
    /// The array domain `domain` is, or `None` for the plain domain.
    pub fn of(domain: Domain) -> Option<Self> {
        match domain {
            Domain::Corner => Some(ArrayDomain::Corner),
            Domain::Face => Some(ArrayDomain::Face),
            Domain::Plain => None,
            Domain::Swatch => Some(ArrayDomain::Swatch),
            Domain::Voxel => Some(ArrayDomain::Voxel),
        }
    }
}

impl Display for ArrayDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            ArrayDomain::Corner => "corner",
            ArrayDomain::Face => "face",
            ArrayDomain::Swatch => "swatch",
            ArrayDomain::Voxel => "voxel",
        })
    }
}

impl From<ArrayDomain> for Domain {
    fn from(domain: ArrayDomain) -> Domain {
        match domain {
            ArrayDomain::Corner => Domain::Corner,
            ArrayDomain::Face => Domain::Face,
            ArrayDomain::Swatch => Domain::Swatch,
            ArrayDomain::Voxel => Domain::Voxel,
        }
    }
}
