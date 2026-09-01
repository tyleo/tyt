/// The glTF material slot a baked map fills.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialSlot {
    /// `pbrMetallicRoughness.baseColorTexture`.
    BaseColor,

    /// `pbrMetallicRoughness.metallicRoughnessTexture`.
    MetallicRoughness,

    /// `occlusionTexture`.
    Occlusion,

    /// One image shared by `occlusionTexture` and
    /// `pbrMetallicRoughness.metallicRoughnessTexture`, the ORM packing.
    OcclusionMetallicRoughness,

    /// `emissiveTexture`.
    Emissive,
}
