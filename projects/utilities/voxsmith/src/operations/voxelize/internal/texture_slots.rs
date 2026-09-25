use crate::operations::voxelize::{MeshInput, TextureSlot};

/// The sampled texture slots of one placed primitive. An unresolved slot
/// leaves its attribute at the material's flat factor.
#[derive(Default)]
pub(crate) struct TextureSlots<'a> {
    /// The base-color texture, an sRGB image sampled into the base color.
    pub base_color: Option<TextureSlot<'a>>,

    /// The metallic-roughness texture, linear data sampled into `metallic`
    /// and `roughness`.
    pub metallic_roughness: Option<TextureSlot<'a>>,

    /// The emissive texture, an sRGB image sampled into the emissive color.
    pub emissive: Option<TextureSlot<'a>>,

    /// The occlusion texture, linear data sampled into the occlusion.
    pub occlusion: Option<TextureSlot<'a>>,
}

impl<'a> TextureSlots<'a> {
    /// The slots of the placed primitive at `primitive` in `input`.
    pub fn resolve(input: &'a MeshInput<'_>, primitive: u32) -> Self {
        let placed = &input.primitives[primitive as usize];
        let material = placed.material;

        let slot = |texture_ref| TextureSlot::resolve(input, placed, texture_ref);

        Self {
            base_color: slot(material.base_color_texture),
            metallic_roughness: slot(material.metallic_roughness_texture),
            emissive: slot(material.emissive_texture),
            occlusion: slot(material.occlusion_texture),
        }
    }

    /// Whether any slot resolved.
    pub fn any(&self) -> bool {
        self.base_color.is_some()
            || self.metallic_roughness.is_some()
            || self.emissive.is_some()
            || self.occlusion.is_some()
    }
}
