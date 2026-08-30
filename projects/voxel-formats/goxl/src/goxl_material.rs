use crate::GoxlDict;

/// A `MATE` material: a PBR description referenced by
/// [`GoxlLayer::material`](crate::GoxlLayer::material).
#[derive(Clone, Debug, PartialEq)]
pub struct GoxlMaterial {
    /// Material name.
    pub name: String,

    /// `[r, g, b, a]` linear base color.
    pub base_color: [f32; 4],

    /// Metallic factor, `0..=1`.
    pub metallic: f32,

    /// Roughness factor, `0..=1`.
    pub roughness: f32,

    /// `[r, g, b]` emission color.
    pub emission: [f32; 3],

    /// Dict keys this crate does not model, preserved verbatim.
    pub extra: GoxlDict,
}

impl Default for GoxlMaterial {
    fn default() -> Self {
        Self {
            name: String::new(),
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.2,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            extra: GoxlDict::default(),
        }
    }
}
