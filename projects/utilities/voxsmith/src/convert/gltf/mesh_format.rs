/// A glTF container this crate meshes into.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshFormat {
    /// glTF text, the `.gltf` file.
    Gltf,

    /// glTF binary, the `.glb` file.
    Glb,
}

impl MeshFormat {
    /// The format whose file extension is `extension`, matched
    /// case-insensitively without the leading dot, or `None` when no format
    /// takes it.
    pub fn from_extension(extension: &str) -> Option<MeshFormat> {
        match extension.to_ascii_lowercase().as_str() {
            "gltf" => Some(MeshFormat::Gltf),
            "glb" => Some(MeshFormat::Glb),
            _ => None,
        }
    }

    /// The file extension, without a leading dot.
    pub fn extension(self) -> &'static str {
        match self {
            MeshFormat::Gltf => "gltf",
            MeshFormat::Glb => "glb",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::MeshFormat;

    #[test]
    fn extensions_round_trip_case_insensitively() {
        for format in [MeshFormat::Gltf, MeshFormat::Glb] {
            assert_eq!(MeshFormat::from_extension(format.extension()), Some(format));
            assert_eq!(
                MeshFormat::from_extension(&format.extension().to_ascii_uppercase()),
                Some(format)
            );
        }
    }

    #[test]
    fn an_unknown_extension_is_none() {
        assert_eq!(MeshFormat::from_extension("obj"), None);
        assert_eq!(MeshFormat::from_extension(""), None);
    }
}
