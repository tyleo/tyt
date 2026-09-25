use meshdoc::MeshMaterial;
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

/// The material one voxel samples, in the vocabulary a palette material
/// carries. Read from a mesh material's flat factors, with the sampled maps
/// applied where a slot resolves. Each distinct material becomes one palette
/// material.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VoxelMaterial {
    /// Straight-RGBA base color in linear light. glTF `baseColorFactor`.
    pub base_color: TyLinSrgbaF64,

    /// Metalness, `0..=1`. glTF `metallicFactor`.
    pub metallic: f64,

    /// Roughness, `0..=1`. glTF `roughnessFactor`.
    pub roughness: f64,

    /// Emissive color in linear light. glTF `emissiveFactor`.
    pub emissive_color: TyLinSrgbF64,

    /// Emissive strength scaling [`emissive_color`](Self::emissive_color), `0+`.
    /// glTF's `KHR_materials_emissive_strength`.
    pub emissive_strength: f64,

    /// Flat ambient occlusion, `0..=1` (`1` = none). glTF `occlusionStrength`.
    pub occlusion: f64,

    /// Index of refraction: `0` for "does not refract", else `1+`. glTF
    /// `KHR_materials_ior`.
    pub ior: f64,

    /// Transmitted fraction, `0..=1`. glTF `KHR_materials_transmission`.
    pub transmission: f64,
}

impl VoxelMaterial {
    /// A flat opaque material of `base_color` with default finish: non-metal,
    /// matte, non-emissive, unoccluded, dielectric, opaque. This is the whole
    /// body in flat mode, and the invented interior a fill color paints.
    pub fn flat(base_color: TyLinSrgbaF64) -> Self {
        Self {
            base_color,
            metallic: 0.0,
            roughness: 1.0,
            emissive_color: TyLinSrgbF64::new(0.0, 0.0, 0.0),
            emissive_strength: 0.0,
            occlusion: 1.0,
            ior: 1.5,
            transmission: 0.0,
        }
    }
}

/// A mesh material's flat factors. Occlusion has no flat factor and starts
/// full.
impl From<&MeshMaterial> for VoxelMaterial {
    fn from(material: &MeshMaterial) -> Self {
        Self {
            base_color: material.base_color_factor,
            metallic: material.metallic_factor,
            roughness: material.roughness_factor,
            emissive_color: material.emissive_factor,
            emissive_strength: material.emissive_strength,
            occlusion: 1.0,
            ior: material.ior,
            transmission: material.transmission_factor,
        }
    }
}
