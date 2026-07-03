use crate::{MeshBaseColorMap, MeshEmissiveMap, MeshMetallicRoughnessMap, MeshOcclusionMap};

/// The optional texture bindings of one mesh material, parallel to the mesh's
/// material table. An absent map leaves its attribute to the material's flat
/// factor; a present map is sampled per texel. The maps that read a texture are
/// base color, metallic-roughness, emissive, and occlusion.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MeshMaterialMaps {
    /// The base-color texture, sampled into `rgba`.
    pub base_color: Option<MeshBaseColorMap>,

    /// The metallic-roughness texture, sampled into `metallic` and `roughness`.
    pub metallic_roughness: Option<MeshMetallicRoughnessMap>,

    /// The emissive texture, sampled into `emissive`.
    pub emissive: Option<MeshEmissiveMap>,

    /// The occlusion texture, sampled into `occlusion`.
    pub occlusion: Option<MeshOcclusionMap>,
}

impl MeshMaterialMaps {
    /// Whether the material carries any texture map, so per-texel sampling has
    /// something to read.
    pub fn any(&self) -> bool {
        self.base_color.is_some()
            || self.metallic_roughness.is_some()
            || self.emissive.is_some()
            || self.occlusion.is_some()
    }
}
