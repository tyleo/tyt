/// One PBR texture map to attach to a test quad.
pub(crate) enum MapSpec<'a> {
    BaseColor {
        png: &'a [u8],
        stream: u32,
        factor: [f64; 4],
    },
    MetallicRoughness {
        png: &'a [u8],
        stream: u32,
        metallic: f64,
        roughness: f64,
    },
    Emissive {
        png: &'a [u8],
        stream: u32,
        factor: [f64; 3],
    },
    Occlusion {
        png: &'a [u8],
        stream: u32,
        strength: f64,
    },
}

impl MapSpec<'_> {
    /// The map's PNG bytes.
    pub(crate) fn png(&self) -> &[u8] {
        match self {
            MapSpec::BaseColor { png, .. }
            | MapSpec::MetallicRoughness { png, .. }
            | MapSpec::Emissive { png, .. }
            | MapSpec::Occlusion { png, .. } => png,
        }
    }
}
