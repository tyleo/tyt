use crate::CliValue;
use meshconv::ReadFormat;

impl CliValue for ReadFormat {
    const VARIANTS: &'static [Self] = &[ReadFormat::Gltf];

    fn name(self) -> &'static str {
        ReadFormat::name(self)
    }

    fn help(self) -> &'static str {
        match self {
            ReadFormat::Gltf => "glTF 2.0, the `.glb` and `.gltf` files",
        }
    }
}
