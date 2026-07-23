/// An object's axis-aligned bounding-box payload, components
/// precision-formatted in Blender.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HierarchyBounds {
    /// The minimum corner components.
    pub min: [String; 3],

    /// The maximum corner components.
    pub max: [String; 3],
}
