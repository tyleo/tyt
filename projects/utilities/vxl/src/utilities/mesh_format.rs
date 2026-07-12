use clap::ValueEnum;
use std::path::Path;

/// A mesh file format that vxl can read or write.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MeshFormat {
    /// glTF text, the `.gltf` file.
    #[value(name = "gltf")]
    Gltf,

    /// glTF binary, the `.glb` file.
    #[value(name = "glb")]
    Glb,
}

impl MeshFormat {
    /// Infers the format from `path`'s extension, matched case-insensitively,
    /// or `None` when the extension names no supported mesh format.
    pub fn from_path(path: &Path) -> Option<MeshFormat> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        match extension.as_str() {
            "gltf" => Some(MeshFormat::Gltf),
            "glb" => Some(MeshFormat::Glb),
            _ => None,
        }
    }

    /// The file extension for this format, without a leading dot.
    pub fn extension(self) -> &'static str {
        match self {
            MeshFormat::Gltf => "gltf",
            MeshFormat::Glb => "glb",
        }
    }
}
