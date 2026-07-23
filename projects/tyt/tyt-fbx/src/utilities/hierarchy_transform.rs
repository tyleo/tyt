/// An object's transform payload, components precision-formatted in
/// Blender.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyTransform {
    /// The position components.
    pub position: [String; 3],

    /// The euler rotation components.
    pub rotation: [String; 3],

    /// The scale components.
    pub scale: [String; 3],
}
