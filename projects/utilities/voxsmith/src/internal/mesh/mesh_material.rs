use ty_math::TySrgbaColor;

/// A mesh material's flat PBR factors in the glTF metallic-roughness attribute
/// vocabulary a voxel palette material carries: `baseColorFactor`,
/// `metallicFactor`, `roughnessFactor`, `emissiveFactor`, `emissiveStrength`,
/// `occlusionStrength`, `ior`, and `transmissionFactor`. This is the
/// per-primitive material: one material per mesh material, read from its
/// factors alone. The voxelizer turns each distinct material into one palette
/// material over value pools bound by these names.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshMaterial {
    /// Straight-RGBA base color in the sRGB storage encoding. glTF
    /// `baseColorFactor`.
    pub base_color: TySrgbaColor,

    /// Metalness, `0..=1`. glTF `metallicFactor`.
    pub metallic: f64,

    /// Roughness, `0..=1`. glTF `roughnessFactor`.
    pub roughness: f64,

    /// Emissive color in the sRGB storage encoding. glTF `emissiveFactor` carries
    /// no alpha, so the alpha is held opaque and ignored.
    pub emissive_factor: TySrgbaColor,

    /// Emissive strength scaling [`emissive_factor`](Self::emissive_factor), `0+`.
    /// glTF's `KHR_materials_emissive_strength`.
    pub emissive_strength: f64,

    /// Flat ambient occlusion, `0..=1` (`1` = none). glTF `occlusionStrength`.
    pub occlusion: f64,

    /// Index of refraction, `1+`. glTF `KHR_materials_ior`.
    pub ior: f64,

    /// Transmitted fraction, `0..=1`. glTF `KHR_materials_transmission`.
    pub transmission: f64,
}

impl MeshMaterial {
    /// A flat opaque material of `base_color` with default finish: non-metal,
    /// matte, non-emissive, unoccluded, dielectric, opaque. This is the whole
    /// body in flat mode, and the invented interior a fill color paints.
    pub fn flat(base_color: TySrgbaColor) -> Self {
        Self {
            base_color,
            metallic: 0.0,
            roughness: 1.0,
            emissive_factor: TySrgbaColor::new(0, 0, 0, 255),
            emissive_strength: 0.0,
            occlusion: 1.0,
            ior: 1.5,
            transmission: 0.0,
        }
    }
}
