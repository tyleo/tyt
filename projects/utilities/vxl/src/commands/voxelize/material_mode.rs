use crate::CliValue;
use voxsmith::MaterialMode;

impl CliValue for MaterialMode {
    const VARIANTS: &'static [Self] = &[
        MaterialMode::Auto,
        MaterialMode::PerPrimitive,
        MaterialMode::PerTexel,
        MaterialMode::Flat,
    ];

    fn name(self) -> &'static str {
        match self {
            MaterialMode::Auto => "auto",
            MaterialMode::PerPrimitive => "per-primitive",
            MaterialMode::PerTexel => "per-texel",
            MaterialMode::Flat => "flat",
        }
    }

    fn help(self) -> &'static str {
        match self {
            MaterialMode::Auto => {
                "Sample per-texel when the mesh carries textures, else per-primitive"
            }
            MaterialMode::PerPrimitive => {
                "One material per glTF material, read from its flat PBR factors"
            }
            MaterialMode::PerTexel => "Sample the material maps at each voxel's surface point",
            MaterialMode::Flat => "Ignore the mesh's materials and paint the one `--fill-color`",
        }
    }
}
