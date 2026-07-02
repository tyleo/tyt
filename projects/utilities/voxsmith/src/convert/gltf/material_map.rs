use crate::{MaterialBake, MaterialSlot};

/// One baked material map: what it writes, the glTF slot it fills, and the
/// relative file name it takes when written externally (also the key it is
/// listed under in the material's `extras`).
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialMap {
    /// The map's relative file name, e.g. `model-albedo.png`, used as the
    /// external URI and the `extras` key.
    pub name: String,

    /// The glTF material slot this map fills.
    pub slot: MaterialSlot,

    /// What the map writes into its image.
    pub bake: MaterialBake,
}
