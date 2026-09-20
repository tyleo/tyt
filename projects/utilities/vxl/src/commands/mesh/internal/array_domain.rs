use crate::CliValue;
use voxsmith::operations::mesh::ArrayDomain;

impl CliValue for ArrayDomain {
    const VARIANTS: &'static [Self] = &[
        ArrayDomain::Corner,
        ArrayDomain::Face,
        ArrayDomain::Swatch,
        ArrayDomain::Voxel,
    ];

    fn name(self) -> &'static str {
        match self {
            ArrayDomain::Corner => "corner",
            ArrayDomain::Face => "face",
            ArrayDomain::Swatch => "swatch",
            ArrayDomain::Voxel => "voxel",
        }
    }

    fn help(self) -> &'static str {
        match self {
            ArrayDomain::Corner => "One entry per face corner",
            ArrayDomain::Face => "One entry per emitted face",
            ArrayDomain::Swatch => "One entry per swatch",
            ArrayDomain::Voxel => "One entry per solid voxel",
        }
    }
}
