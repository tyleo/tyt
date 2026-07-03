use ty_math::TyVector2F64;

/// A triangle's per-vertex texture coordinates, one optional set per PBR map
/// slot. Each slot holds the coordinates of the TEXCOORD set that map declares,
/// so a map that names its own set samples the right coordinates. A slot is
/// `None` when the material lacks that map or the primitive carried no
/// coordinates for its set.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct MeshTriangleUvs {
    /// The base-color map's coordinates.
    pub base_color: Option<[TyVector2F64; 3]>,

    /// The metallic-roughness map's coordinates.
    pub metallic_roughness: Option<[TyVector2F64; 3]>,

    /// The emissive map's coordinates.
    pub emissive: Option<[TyVector2F64; 3]>,

    /// The occlusion map's coordinates.
    pub occlusion: Option<[TyVector2F64; 3]>,
}
