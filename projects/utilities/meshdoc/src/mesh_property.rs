use crate::MeshPropertyValue;
use std::collections::HashSet;

/// A named property outside the modeled fields of a
/// [`MeshMaterial`](crate::MeshMaterial) or a
/// [`MeshObject`](crate::MeshObject), so a bridge and a producer have a
/// typed home for a value the model does not place. The name is unique
/// within its owner.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshProperty {
    /// The property name.
    pub name: String,

    /// The property value.
    pub value: MeshPropertyValue,
}

/// The name of the first property in `properties` that repeats an earlier
/// one.
pub(crate) fn first_duplicate_property_name(properties: &[MeshProperty]) -> Option<&str> {
    let mut seen = HashSet::with_capacity(properties.len());

    properties
        .iter()
        .map(|property| property.name.as_str())
        .find(|name| !seen.insert(*name))
}

/// The name of the first property in `properties` holding a non-finite
/// float.
pub(crate) fn first_non_finite_property(properties: &[MeshProperty]) -> Option<&str> {
    properties
        .iter()
        .find(|property| !property.value.is_finite())
        .map(|property| property.name.as_str())
}
