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
