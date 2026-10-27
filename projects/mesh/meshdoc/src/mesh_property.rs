use crate::MeshPropertyValue;

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
