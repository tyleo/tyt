/// The glTF material slot a baked map fills. The caller picks the slot from the
/// map's meaning; a map with no standard slot is referenced only through the
/// material's `extras`, so a custom pipeline can still find it.
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

    /// No standard glTF slot; the map is emitted and referenced only from the
    /// material's `extras`.
    None,
}
