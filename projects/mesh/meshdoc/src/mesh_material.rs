use crate::{
    BMeshFile, BMeshTexture, MeshAlphaMode, MeshProperty, MeshPropertyValue, MeshTextureRef,
    material::{
        ALPHA_CUTOFF, ALPHA_CUTOFF_DEFAULT, BASE_COLOR, COLOR_RANGE, EMISSIVE_COLOR,
        EMISSIVE_STRENGTH, EMISSIVE_STRENGTH_DEFAULT, IOR, IOR_DEFAULT, METALLIC, METALLIC_DEFAULT,
        NORMAL_SCALE, NORMAL_SCALE_DEFAULT, OCCLUSION_STRENGTH, OCCLUSION_STRENGTH_DEFAULT,
        ROUGHNESS, ROUGHNESS_DEFAULT, TRANSMISSION, TRANSMISSION_DEFAULT, scalar_range,
    },
};
use branded_id::{U32Id, soa::IdRemap};
use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

/// A metallic-roughness material, the model the popular mesh formats share.
///
/// Every factor is in linear light and holds to the range its name in the
/// `material` module fixes. A texture slot pairs a texture with the UV stream
/// the drawing primitives sample it through; the texture's texel combines
/// with the factor beside it. The ids reference a
/// [`MeshMain`](crate::MeshMain) and are meaningful only within it.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshMaterial {
    /// Display name.
    pub name: String,

    /// The straight-alpha base color factor.
    pub base_color_factor: TyLinSrgbaF64,

    /// The base color texture, an sRGB image the factor multiplies.
    pub base_color_texture: Option<MeshTextureRef>,

    /// The metalness factor.
    pub metallic_factor: f64,

    /// The roughness factor.
    pub roughness_factor: f64,

    /// The metallic-roughness texture: roughness in green, metalness in
    /// blue, each scaled by its factor.
    pub metallic_roughness_texture: Option<MeshTextureRef>,

    /// The tangent-space normal texture.
    pub normal_texture: Option<MeshTextureRef>,

    /// The scale applied to the normal texture's `x` and `y`.
    pub normal_scale: f64,

    /// The occlusion texture, occlusion in red.
    pub occlusion_texture: Option<MeshTextureRef>,

    /// How far the occlusion texture darkens from full.
    pub occlusion_strength: f64,

    /// The emissive color factor.
    pub emissive_factor: TyLinSrgbF64,

    /// The emissive texture, an sRGB image the factor multiplies.
    pub emissive_texture: Option<MeshTextureRef>,

    /// The strength scaling the emissive color.
    pub emissive_strength: f64,

    /// How the base color alpha renders.
    pub alpha_mode: MeshAlphaMode,

    /// The alpha cutoff under [`MeshAlphaMode::Mask`].
    pub alpha_cutoff: f64,

    /// Whether back faces render.
    pub double_sided: bool,

    /// The index of refraction: `0` for "does not refract", else `1..`.
    pub ior: f64,

    /// The transmitted fraction of light.
    pub transmission_factor: f64,

    /// The transmission texture, transmission in red, scaled by the factor.
    pub transmission_texture: Option<MeshTextureRef>,

    /// Named properties outside the modeled set, names unique.
    pub properties: Vec<MeshProperty>,
}

impl MeshMaterial {
    /// A material named `name` with every factor at its standard default and
    /// no textures or properties: opaque white, fully metallic and rough.
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_color_factor: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0),
            base_color_texture: None,
            metallic_factor: METALLIC_DEFAULT,
            roughness_factor: ROUGHNESS_DEFAULT,
            metallic_roughness_texture: None,
            normal_texture: None,
            normal_scale: NORMAL_SCALE_DEFAULT,
            occlusion_texture: None,
            occlusion_strength: OCCLUSION_STRENGTH_DEFAULT,
            emissive_factor: TyLinSrgbF64::new(0.0, 0.0, 0.0),
            emissive_texture: None,
            emissive_strength: EMISSIVE_STRENGTH_DEFAULT,
            alpha_mode: MeshAlphaMode::Opaque,
            alpha_cutoff: ALPHA_CUTOFF_DEFAULT,
            double_sided: false,
            ior: IOR_DEFAULT,
            transmission_factor: TRANSMISSION_DEFAULT,
            transmission_texture: None,
            properties: Vec::new(),
        }
    }

    /// The scalar factor named `key` in the `material` vocabulary, or `None`
    /// for a key outside it.
    pub fn scalar(&self, key: &str) -> Option<f64> {
        match key {
            METALLIC => Some(self.metallic_factor),
            ROUGHNESS => Some(self.roughness_factor),
            OCCLUSION_STRENGTH => Some(self.occlusion_strength),
            NORMAL_SCALE => Some(self.normal_scale),
            EMISSIVE_STRENGTH => Some(self.emissive_strength),
            IOR => Some(self.ior),
            TRANSMISSION => Some(self.transmission_factor),
            ALPHA_CUTOFF => Some(self.alpha_cutoff),
            _ => None,
        }
    }

    /// Sets the scalar factor named `key` in the `material` vocabulary,
    /// reporting whether `key` matched one. The range is checked when the
    /// material enters a [`MeshMain`](crate::MeshMain).
    pub fn set_scalar(&mut self, key: &str, value: f64) -> bool {
        let slot = match key {
            METALLIC => &mut self.metallic_factor,
            ROUGHNESS => &mut self.roughness_factor,
            OCCLUSION_STRENGTH => &mut self.occlusion_strength,
            NORMAL_SCALE => &mut self.normal_scale,
            EMISSIVE_STRENGTH => &mut self.emissive_strength,
            IOR => &mut self.ior,
            TRANSMISSION => &mut self.transmission_factor,
            ALPHA_CUTOFF => &mut self.alpha_cutoff,
            _ => return false,
        };

        *slot = value;
        true
    }

    /// The property named `name`, or `None` when the material has none.
    pub fn property(&self, name: &str) -> Option<&MeshProperty> {
        self.properties
            .iter()
            .find(|property| property.name == name)
    }

    /// Every texture reference the material draws, the slots first and then
    /// the texture-valued properties, in listing order.
    pub fn iter_texture_refs(&self) -> impl Iterator<Item = MeshTextureRef> + '_ {
        [
            self.base_color_texture,
            self.metallic_roughness_texture,
            self.normal_texture,
            self.occlusion_texture,
            self.emissive_texture,
            self.transmission_texture,
        ]
        .into_iter()
        .flatten()
        .chain(
            self.properties
                .iter()
                .filter_map(|property| property.value.texture_ref()),
        )
    }

    /// Every file the material's properties point at, in listing order.
    pub fn iter_file_ids(&self) -> impl Iterator<Item = U32Id<BMeshFile>> + '_ {
        self.properties
            .iter()
            .filter_map(|property| property.value.file_id())
    }

    /// Rewrites every texture reference through `remap`, the texture id
    /// pool's relabeling from a [`MeshMain::gc`](crate::MeshMain::gc).
    /// Requires a referentially valid material, so every translation
    /// resolves.
    pub(crate) fn relabel_textures(&mut self, remap: &IdRemap<BMeshTexture, u32>) {
        let relabel = |slot: &mut Option<MeshTextureRef>| {
            if let Some(texture_ref) = slot {
                texture_ref.texture_id = remap
                    .new_id(texture_ref.texture_id)
                    .expect("a material draws a live texture in a valid state");
            }
        };

        relabel(&mut self.base_color_texture);
        relabel(&mut self.metallic_roughness_texture);
        relabel(&mut self.normal_texture);
        relabel(&mut self.occlusion_texture);
        relabel(&mut self.emissive_texture);
        relabel(&mut self.transmission_texture);

        for property in &mut self.properties {
            if let MeshPropertyValue::Texture(texture_ref) = &mut property.value {
                texture_ref.texture_id = remap
                    .new_id(texture_ref.texture_id)
                    .expect("a material property draws a live texture in a valid state");
            }
        }
    }

    /// Rewrites every file reference through `remap`, the file id pool's
    /// relabeling from a [`MeshMain::gc`](crate::MeshMain::gc). Requires a
    /// referentially valid material, so every translation resolves.
    pub(crate) fn relabel_files(&mut self, remap: &IdRemap<BMeshFile, u32>) {
        for property in &mut self.properties {
            if let MeshPropertyValue::File(file_id) = &mut property.value {
                *file_id = remap
                    .new_id(*file_id)
                    .expect("a material property points at a live file in a valid state");
            }
        }
    }

    /// The first factor outside its range, as its vocabulary name and value.
    /// A color reports its first component outside [`COLOR_RANGE`].
    pub(crate) fn first_factor_out_of_range(&self) -> Option<(&'static str, f64)> {
        let colors = [
            (
                BASE_COLOR,
                <[f64; 4]>::from(self.base_color_factor).to_vec(),
            ),
            (
                EMISSIVE_COLOR,
                <[f64; 3]>::from(self.emissive_factor).to_vec(),
            ),
        ];

        for (name, components) in colors {
            if let Some(&component) = components
                .iter()
                .find(|&&component| !COLOR_RANGE.contains(component))
            {
                return Some((name, component));
            }
        }

        [
            METALLIC,
            ROUGHNESS,
            OCCLUSION_STRENGTH,
            NORMAL_SCALE,
            EMISSIVE_STRENGTH,
            IOR,
            TRANSMISSION,
            ALPHA_CUTOFF,
        ]
        .into_iter()
        .find_map(|name| {
            let value = self.scalar(name).expect("every listed name is a scalar");
            let range = scalar_range(name).expect("every listed name has a range");

            (!range.contains(value)).then_some((name, value))
        })
    }
}

impl Default for MeshMaterial {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        MeshMaterial, MeshProperty, MeshPropertyValue, MeshTextureRef,
        material::{IOR, METALLIC, ROUGHNESS},
    };
    use branded_id::U32Id;
    use ty_math::TyLinSrgbaF64;

    #[test]
    fn scalars_read_and_write_by_vocabulary_name() {
        let mut material = MeshMaterial::default();

        assert_eq!(material.scalar(METALLIC), Some(1.0));
        assert!(material.set_scalar(ROUGHNESS, 0.25));
        assert_eq!(material.roughness_factor, 0.25);
        assert!(!material.set_scalar("subsurface", 0.5));
        assert_eq!(material.scalar("subsurface"), None);
    }

    #[test]
    fn the_first_out_of_range_factor_is_named() {
        let mut material = MeshMaterial::default();
        assert_eq!(material.first_factor_out_of_range(), None);

        material.ior = 0.5;
        assert_eq!(material.first_factor_out_of_range(), Some((IOR, 0.5)));

        material.ior = 0.0;
        assert_eq!(material.first_factor_out_of_range(), None);

        material.base_color_factor = TyLinSrgbaF64::new(1.5, 0.0, 0.0, 1.0);
        assert_eq!(
            material.first_factor_out_of_range(),
            Some(("baseColor", 1.5))
        );
    }

    #[test]
    fn texture_refs_list_the_slots_then_the_properties() {
        let slot = MeshTextureRef {
            texture_id: U32Id::from_u32(0),
            uv_stream_id: U32Id::from_u32(0),
        };
        let property = MeshTextureRef {
            texture_id: U32Id::from_u32(1),
            uv_stream_id: U32Id::from_u32(2),
        };

        let material = MeshMaterial {
            emissive_texture: Some(slot),
            properties: vec![
                MeshProperty {
                    name: "count".to_owned(),
                    value: MeshPropertyValue::Int(3),
                },
                MeshProperty {
                    name: "detail".to_owned(),
                    value: MeshPropertyValue::Texture(property),
                },
            ],
            ..Default::default()
        };

        assert_eq!(
            material.iter_texture_refs().collect::<Vec<_>>(),
            [slot, property]
        );
    }
}
