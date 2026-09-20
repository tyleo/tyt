use crate::commands::BoundNames;
use serde::Deserialize;
use voxsmith::operations::mesh::ArrayDomain;

/// A profile's `computeIndex`, each domain key holding its bound names.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ComputeIndexEntry {
    pub(crate) corner: BoundNames,
    pub(crate) face: BoundNames,
    pub(crate) swatch: BoundNames,
    pub(crate) voxel: BoundNames,
}

impl ComputeIndexEntry {
    /// Every bound name with its domain, in key order.
    pub(crate) fn bindings(&self) -> impl Iterator<Item = (ArrayDomain, &str)> {
        [
            (ArrayDomain::Corner, &self.corner),
            (ArrayDomain::Face, &self.face),
            (ArrayDomain::Swatch, &self.swatch),
            (ArrayDomain::Voxel, &self.voxel),
        ]
        .into_iter()
        .flat_map(|(domain, names)| names.0.iter().map(move |name| (domain, name.as_str())))
    }
}
