/// The glTF container a document is written as.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GltfContainer {
    /// Binary glTF, a `.glb` file holding its buffer as a chunk.
    #[default]
    Glb,

    /// JSON glTF, a `.gltf` file holding its buffer as a data URI.
    Gltf,
}

impl GltfContainer {
    /// The container a file extension implies, matched case-insensitively,
    /// or `None` for any other extension.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "glb" => Some(GltfContainer::Glb),
            "gltf" => Some(GltfContainer::Gltf),
            _ => None,
        }
    }

    /// The extension a document in this container takes.
    pub fn extension(self) -> &'static str {
        match self {
            GltfContainer::Glb => "glb",
            GltfContainer::Gltf => "gltf",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gltf::GltfContainer;

    #[test]
    fn extensions_imply_a_container() {
        assert_eq!(
            GltfContainer::from_extension("GLB"),
            Some(GltfContainer::Glb)
        );

        assert_eq!(
            GltfContainer::from_extension("gltf"),
            Some(GltfContainer::Gltf)
        );

        assert_eq!(GltfContainer::from_extension("obj"), None);

        assert_eq!(GltfContainer::Gltf.extension(), "gltf");
    }
}
