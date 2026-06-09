use clap::ValueEnum;

/// A 3D file format Meshy can include in a task's output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TargetFormat {
    /// glTF binary.
    Glb,

    /// Wavefront OBJ.
    Obj,

    /// Autodesk FBX.
    Fbx,

    /// Stereolithography.
    Stl,

    /// Universal Scene Description (zipped).
    Usdz,

    /// 3D Manufacturing Format.
    #[value(name = "3mf")]
    ThreeMf,
}

impl TargetFormat {
    /// Returns the string sent in the API's `target_formats`.
    pub fn as_api_str(self) -> &'static str {
        match self {
            TargetFormat::Glb => "glb",
            TargetFormat::Obj => "obj",
            TargetFormat::Fbx => "fbx",
            TargetFormat::Stl => "stl",
            TargetFormat::Usdz => "usdz",
            TargetFormat::ThreeMf => "3mf",
        }
    }
}
