use crate::{BSdfShades, SdfProperty};
use branded_id::U32Id;

/// A material: an entry of [`SdfState::materials`](crate::SdfState::materials).
#[derive(Clone, Debug, PartialEq)]
pub enum SdfMaterial {
    /// A material holding its properties. No two properties share a name.
    Material { properties: Vec<SdfProperty> },

    /// The shade at `index` of a shades entry, counting from the darkest.
    Shade {
        shades_id: U32Id<BSdfShades>,

        index: u32,
    },
}
