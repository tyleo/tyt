use crate::{AttributeBinding, AttributeType, ColorComponent, Error, Result};
use std::{result::Result as StdResult, str::FromStr};
use voxsmith::ROUGHNESS_FACTOR;

/// One channel's value in a material map: an attribute by name, optionally one
/// color component of it, optionally inverted as `1-<attribute>`, the constant
/// `0` or `1`, or `computed-occlusion` derived from the voxel geometry.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelSource {
    /// An attribute value by name, optionally one color `component` of it,
    /// inverted to `1 - value` when `invert` is set.
    Attribute {
        key: String,
        component: Option<ColorComponent>,
        invert: bool,
    },
    /// The constant `0`.
    Zero,
    /// The constant `1`.
    One,
    /// Occlusion computed from the voxel geometry. Always bakes into an unwrap
    /// layout.
    ComputedOcclusion,
}

impl ChannelSource {
    /// Resolves this source's attribute key against the `--define-attribute`
    /// bindings and validates its color component against the key's type,
    /// rejecting `computed-occlusion` under the palette atlas.
    pub(crate) fn resolve(&self, bindings: &[AttributeBinding]) -> Result<ChannelSource> {
        let ChannelSource::Attribute {
            key,
            component,
            invert,
        } = self
        else {
            if let ChannelSource::ComputedOcclusion = self {
                return Err(ChannelSource::computed_occlusion_unsupported());
            }
            return Ok(self.clone());
        };

        // A binding gives the key a concrete voxel attribute key and a kind; a
        // bare key resolves to itself, a color only when it is a built-in glTF
        // color, else a scalar.
        let (resolved_key, ty) = match bindings
            .iter()
            .find(|binding| binding.name() == key.as_str())
        {
            Some(binding) => (binding.key().to_string(), binding.ty()),
            None => match AttributeType::builtin_color(key) {
                Some(ty) => (key.clone(), ty),
                None => (key.clone(), AttributeType::Float),
            },
        };

        if ty.is_color() {
            match component {
                None => {
                    return Err(Error::usage(format!(
                        "`{key}` is a color; name a component, as `{key}.r`"
                    )));
                }
                Some(ColorComponent::A) if !ty.has_alpha() => {
                    return Err(Error::usage(format!(
                        "`{key}` is a color with no alpha; use r, g, or b"
                    )));
                }
                _ => {}
            }
        } else if component.is_some() {
            return Err(Error::usage(format!(
                "`{key}` is a scalar and has no color component"
            )));
        }

        Ok(ChannelSource::Attribute {
            key: resolved_key,
            component: *component,
            invert: *invert,
        })
    }

    /// The shared rejection for `computed-occlusion`, which needs the
    /// not-yet-supported unwrap layout. Both the `--texture` preset and a
    /// `--texture-map` channel reject through here; it fires only under the
    /// palette atlas, since the `--atlas unwrap` gate runs first.
    pub(crate) fn computed_occlusion_unsupported() -> Error {
        Error::usage(
            "computed-occlusion needs the unwrap atlas layout, which is not yet supported; \
             use --atlas palette",
        )
    }
}

impl FromStr for ChannelSource {
    type Err = String;

    /// Parses one channel expression. `0` and `1` are the constants,
    /// `computed-occlusion` the geometry-derived occlusion, a leading `1-`
    /// inverts an attribute, a trailing `.r`/`.g`/`.b`/`.a` reads one color
    /// component, and `smoothness` canonicalizes to inverted `roughnessFactor`.
    fn from_str(value: &str) -> StdResult<Self, Self::Err> {
        match value {
            "0" => return Ok(ChannelSource::Zero),
            "1" => return Ok(ChannelSource::One),
            "computed-occlusion" => return Ok(ChannelSource::ComputedOcclusion),
            _ => {}
        }
        let (body, invert) = match value.strip_prefix("1-") {
            Some(rest) => (rest, true),
            None => (value, false),
        };
        // A trailing `.r`/`.g`/`.b`/`.a` selects one color component; a longer
        // suffix is part of the name, so dotted keys pass through unsplit.
        let (name, component) = match body.rsplit_once('.') {
            Some((head, tail))
                if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                (head, Some(tail.parse::<ColorComponent>()?))
            }
            _ => (body, None),
        };
        if name.is_empty() {
            return Err(format!("`{value}` names no attribute"));
        }
        // `smoothness` is the derived `1-roughnessFactor`, so reading it flips
        // the inversion against the stored `roughnessFactor` attribute.
        let (key, invert) = if name == "smoothness" {
            (ROUGHNESS_FACTOR.to_string(), !invert)
        } else {
            (name.to_string(), invert)
        };
        Ok(ChannelSource::Attribute {
            key,
            component,
            invert,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttributeBinding, AttributeType, ChannelSource, ColorComponent};
    use voxsmith::{BASE_COLOR_FACTOR, EMISSIVE_FACTOR, METALLIC_FACTOR, ROUGHNESS_FACTOR};

    fn attribute(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    fn attribute_with(key: &str, component: Option<ColorComponent>, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component,
            invert,
        }
    }

    fn component(key: &str, component: ColorComponent, invert: bool) -> ChannelSource {
        ChannelSource::Attribute {
            key: key.to_string(),
            component: Some(component),
            invert,
        }
    }

    /// A scalar, a four-component color, and a three-component color binding, so
    /// resolution has one of each kind to exercise.
    fn bindings() -> Vec<AttributeBinding> {
        vec![
            AttributeBinding::new(
                "gloss".to_owned(),
                ROUGHNESS_FACTOR.to_owned(),
                AttributeType::Float,
            ),
            AttributeBinding::new("tint".to_owned(), "tint".to_owned(), AttributeType::Srgba),
            AttributeBinding::new("glow".to_owned(), "glow".to_owned(), AttributeType::Srgb),
        ]
    }

    #[test]
    fn parses_constants_and_computed_occlusion() {
        assert_eq!("0".parse::<ChannelSource>().unwrap(), ChannelSource::Zero);
        assert_eq!("1".parse::<ChannelSource>().unwrap(), ChannelSource::One);
        assert_eq!(
            "computed-occlusion".parse::<ChannelSource>().unwrap(),
            ChannelSource::ComputedOcclusion
        );
    }

    #[test]
    fn parses_attribute_and_inverse() {
        assert_eq!(
            "metallicFactor".parse::<ChannelSource>().unwrap(),
            attribute(METALLIC_FACTOR, false)
        );
        assert_eq!(
            "1-metallicFactor".parse::<ChannelSource>().unwrap(),
            attribute(METALLIC_FACTOR, true)
        );
    }

    #[test]
    fn smoothness_canonicalizes_to_inverted_roughness() {
        assert_eq!(
            "roughnessFactor".parse::<ChannelSource>().unwrap(),
            attribute(ROUGHNESS_FACTOR, false)
        );
        assert_eq!(
            "1-roughnessFactor".parse::<ChannelSource>().unwrap(),
            attribute(ROUGHNESS_FACTOR, true)
        );
        assert_eq!(
            "smoothness".parse::<ChannelSource>().unwrap(),
            attribute(ROUGHNESS_FACTOR, true)
        );
        assert_eq!(
            "1-smoothness".parse::<ChannelSource>().unwrap(),
            attribute(ROUGHNESS_FACTOR, false)
        );
    }

    #[test]
    fn parses_color_components() {
        assert_eq!(
            "baseColorFactor.r".parse::<ChannelSource>().unwrap(),
            component(BASE_COLOR_FACTOR, ColorComponent::R, false)
        );
        assert_eq!(
            "1-baseColorFactor.a".parse::<ChannelSource>().unwrap(),
            component(BASE_COLOR_FACTOR, ColorComponent::A, true)
        );
    }

    #[test]
    fn keeps_dotted_keys_without_a_component() {
        assert_eq!(
            "my.attr".parse::<ChannelSource>().unwrap(),
            attribute("my.attr", false)
        );
    }

    #[test]
    fn rejects_empty_attribute() {
        assert!("".parse::<ChannelSource>().is_err());
        assert!("1-".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn rejects_unknown_color_component() {
        assert!("baseColorFactor.z".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn a_binding_resolves_to_its_concrete_key() {
        // The scalar binding `gloss` reads the layer's `roughnessFactor`.
        let resolved = attribute("gloss", true).resolve(&bindings()).unwrap();
        assert_eq!(resolved, attribute(ROUGHNESS_FACTOR, true));

        // The color binding `tint` reads a component of the layer's `tint`.
        let resolved = component("tint", ColorComponent::R, false)
            .resolve(&bindings())
            .unwrap();
        assert_eq!(resolved, component("tint", ColorComponent::R, false));
    }

    #[test]
    fn a_bare_scalar_and_base_color_validate_their_components() {
        // A scalar takes no component; `baseColorFactor` is a built-in color.
        assert!(
            attribute(METALLIC_FACTOR, false)
                .resolve(&bindings())
                .is_ok()
        );
        assert!(
            component(METALLIC_FACTOR, ColorComponent::R, false)
                .resolve(&bindings())
                .is_err()
        );
        assert!(
            component(BASE_COLOR_FACTOR, ColorComponent::A, false)
                .resolve(&bindings())
                .is_ok()
        );
        assert!(
            attribute_with(BASE_COLOR_FACTOR, None, false)
                .resolve(&bindings())
                .is_err()
        );
    }

    #[test]
    fn emissive_factor_is_an_alpha_less_built_in_color() {
        // The emissive color is the second built-in color, so a bare channel
        // must name a component; but glTF emissive has no alpha, so `.a` is
        // rejected while `.g` is fine.
        assert!(
            component(EMISSIVE_FACTOR, ColorComponent::G, false)
                .resolve(&bindings())
                .is_ok()
        );
        assert!(
            attribute_with(EMISSIVE_FACTOR, None, false)
                .resolve(&bindings())
                .is_err()
        );
        assert!(
            component(EMISSIVE_FACTOR, ColorComponent::A, false)
                .resolve(&bindings())
                .is_err()
        );
    }

    #[test]
    fn a_three_component_color_rejects_alpha() {
        // `glow` is declared `srgb`, so it has r/g/b but no alpha.
        assert!(
            component("glow", ColorComponent::B, false)
                .resolve(&bindings())
                .is_ok()
        );
        assert!(
            component("glow", ColorComponent::A, false)
                .resolve(&bindings())
                .is_err()
        );
    }

    #[test]
    fn a_color_binding_needs_a_component() {
        assert!(
            attribute_with("tint", None, false)
                .resolve(&bindings())
                .is_err()
        );
    }

    #[test]
    fn constants_pass_and_computed_occlusion_is_rejected() {
        assert_eq!(
            ChannelSource::One.resolve(&bindings()).unwrap(),
            ChannelSource::One
        );
        assert!(
            ChannelSource::ComputedOcclusion
                .resolve(&bindings())
                .is_err()
        );
    }
}
