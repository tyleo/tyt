use crate::operations::mesh::{MaterialBake, MaterialSlot};

/// One baked material map. Its name is the image's file name when written
/// loose and the material property name of a slotless map.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialMap {
    /// The map's image name, e.g. `model-albedo.png`.
    pub name: String,

    /// The material slot this map fills.
    pub slot: MaterialSlot,

    /// What the map writes into its image.
    pub bake: MaterialBake,
}
