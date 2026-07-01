use ty_math::TySrgbaColor;
use voxcore::VoxValue;

/// A mesh material's flat PBR factors, resolved into the Voxel Json attribute
/// vocabulary a voxel palette cell carries: `rgba`, `metallic`, `roughness`,
/// `emissive`, and `occlusion`. This is the per-primitive material: one cell per
/// mesh material, read from its factors alone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MeshMaterial {
    /// Straight-RGBA base color in the sRGB storage encoding.
    pub rgba: TySrgbaColor,

    /// Metalness, `0..=1`.
    pub metallic: f64,

    /// Roughness, `0..=1`.
    pub roughness: f64,

    /// Emissive strength scaling `rgba`, `0+`.
    pub emissive: f64,

    /// Flat ambient occlusion, `0..=1` (`1` = none).
    pub occlusion: f64,
}

/// The attribute keys a voxelized material writes, in the order
/// [`MeshMaterial::cell_values`] returns them.
pub(crate) const MATERIAL_ATTRIBUTES: [&str; 5] =
    ["rgba", "metallic", "roughness", "emissive", "occlusion"];

impl MeshMaterial {
    /// A flat opaque material of `rgba` with default finish: matte
    /// (`roughness 1`), non-metal, non-emissive, unoccluded. This is the whole
    /// body in flat mode, and the invented interior a fill color paints.
    pub fn flat(rgba: TySrgbaColor) -> Self {
        Self {
            rgba,
            metallic: 0.0,
            roughness: 1.0,
            emissive: 0.0,
            occlusion: 1.0,
        }
    }

    /// The `#RRGGBBAA` hex string for [`rgba`](Self::rgba).
    pub fn hex(&self) -> String {
        let TySrgbaColor { r, g, b, a } = self.rgba;
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }

    /// One palette-cell row, one value per key in [`MATERIAL_ATTRIBUTES`] order.
    pub fn cell_values(&self) -> Vec<VoxValue> {
        vec![
            VoxValue::Text(self.hex()),
            VoxValue::Number(self.metallic),
            VoxValue::Number(self.roughness),
            VoxValue::Number(self.emissive),
            VoxValue::Number(self.occlusion),
        ]
    }
}
