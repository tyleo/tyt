/// The material texture slot a baked map fills, chosen by the caller from
/// the map's meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialSlot {
    /// `pbrMetallicRoughness.baseColorTexture`.
    BaseColor,

    /// `pbrMetallicRoughness.metallicRoughnessTexture`.
    MetallicRoughness,

    /// `occlusionTexture`.
    Occlusion,

    /// One image shared by both `occlusionTexture` and
    /// `pbrMetallicRoughness.metallicRoughnessTexture`, the ORM packing whose
    /// channels align with both slots.
    OcclusionMetallicRoughness,

    /// `emissiveTexture`, with the emissive factor set to full so the texture
    /// shows.
    Emissive,

    /// No standard slot; the map rides as a texture property named after the
    /// map, so a custom pipeline can still find it.
    None,
}
