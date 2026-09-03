use crate::CliValue;
use voxsmith::MeshFormat;

impl CliValue for MeshFormat {
    const VARIANTS: &'static [Self] = &[MeshFormat::Gltf, MeshFormat::Glb];

    fn name(self) -> &'static str {
        match self {
            MeshFormat::Gltf => "gltf",
            MeshFormat::Glb => "glb",
        }
    }

    fn help(self) -> &'static str {
        match self {
            MeshFormat::Gltf => "glTF text, the `.gltf` file",
            MeshFormat::Glb => "glTF binary, the `.glb` file",
        }
    }
}
