use crate::{Texture, TextureBake};

/// One resolved material map the `mesh` command hands to the writer: its file
/// name, the preset it came from (for the glTF slot, or `None` for a custom
/// `--texture-map` packing), and the fully resolved bake. Flag-agnostic, so the
/// implementation lowers it into the mesh engine without knowing the flags.
#[derive(Clone, Debug)]
pub struct MeshTextureMap {
    /// The map's relative file name, e.g. `model-albedo.png`.
    pub name: String,

    /// The preset the map came from, which picks its glTF slot; `None` for a
    /// custom `--texture-map` packing, which has no standard slot.
    pub preset: Option<Texture>,

    /// The resolved bake: what the map writes into its image, with any custom
    /// attribute binding already applied.
    pub bake: TextureBake,
}
