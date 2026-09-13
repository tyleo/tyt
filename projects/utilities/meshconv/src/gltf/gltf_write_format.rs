use crate::gltf::{GltfContainer, GltfWriteOptions};

/// A glTF write target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GltfWriteFormat {
    /// The container.
    pub container: GltfContainer,

    /// The writer's options.
    pub options: GltfWriteOptions,
}
