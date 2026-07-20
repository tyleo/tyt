use crate::commands::TextureBake;
use voxsmith::MaterialSlot;

/// One resolved material map the `mesh` command hands to the writer.
/// Flag-agnostic, so the implementation lowers it into the mesh engine without
/// seeing a CLI flag.
#[derive(Clone, Debug)]
pub struct MeshTextureMap {
    /// The map's relative file name, e.g. `model-albedo.png`.
    pub name: String,

    /// The glTF slot the map fills, or `None` for a custom `--texture-map`
    /// packing.
    pub slot: Option<MaterialSlot>,

    /// The resolved bake: what the map writes into its image, with any custom
    /// property binding already applied.
    pub bake: TextureBake,
}
