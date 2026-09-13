use crate::GltfImageStorage;

/// The options [`to_gltf_file`](crate::to_gltf_file) writes under.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GltfWriteOptions {
    /// Where the images go.
    pub images: GltfImageStorage,
}
