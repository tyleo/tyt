use crate::{Error, Result, VectorComponent, commands::PropertyBinding};
use std::{result::Result as StdResult, str::FromStr};

/// One channel's value in a material map: a property by name, optionally one
/// color component of it, optionally inverted as `1-<property>`, the constant
/// `0` or `1`, or `computed-occlusion` derived from the voxel geometry.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelSource {
    /// A property value by name, optionally one color `component` of it,
    /// inverted to `1 - value` when `invert` is set.
    Property {
        key: String,
        component: Option<VectorComponent>,
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
    /// Resolves this source's property key against the `--define-property`
    /// bindings and rejects `computed-occlusion` under the palette atlas. The
    /// color component is validated later, once the document loads, against the
    /// key's value pool kind, since the type lives in the file, not the flags.
    pub(crate) fn resolve(&self, bindings: &[PropertyBinding]) -> Result<ChannelSource> {
        let ChannelSource::Property {
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

        // A binding renames the key to a concrete voxel property key; a bare
        // key resolves to itself.
        let resolved_key = match bindings
            .iter()
            .find(|binding| binding.name() == key.as_str())
        {
            Some(binding) => binding.key().to_string(),
            None => key.clone(),
        };

        Ok(ChannelSource::Property {
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
    /// inverts a property, and a trailing `.r`/`.g`/`.b`/`.a` or
    /// `.x`/`.y`/`.z`/`.w` reads one component. A property reference carries
    /// no whitespace.
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
        // A trailing letter from either alias set selects one component. A
        // longer suffix is part of the name, so dotted keys pass through
        // unsplit.
        let (name, component) = match body.rsplit_once('.') {
            Some((head, tail))
                if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) =>
            {
                (head, Some(tail.parse::<VectorComponent>()?))
            }
            _ => (body, None),
        };
        if name.is_empty() {
            return Err(format!("`{value}` names no property"));
        }
        if name.chars().any(char::is_whitespace) {
            return Err(format!(
                "`{value}` names a property with whitespace; alias it with --define-property"
            ));
        }
        Ok(ChannelSource::Property {
            key: name.to_string(),
            component,
            invert,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        VectorComponent,
        commands::{ChannelSource, PropertyBinding},
    };
    use voxsmith::voxcore::material::{BASE_COLOR, METALLIC, ROUGHNESS};

    fn property(key: &str, invert: bool) -> ChannelSource {
        ChannelSource::Property {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    fn property_with(key: &str, component: Option<VectorComponent>, invert: bool) -> ChannelSource {
        ChannelSource::Property {
            key: key.to_string(),
            component,
            invert,
        }
    }

    fn component(key: &str, component: VectorComponent, invert: bool) -> ChannelSource {
        ChannelSource::Property {
            key: key.to_string(),
            component: Some(component),
            invert,
        }
    }

    /// Two aliases, `gloss` and `tint`, so resolution has a rename to apply.
    fn bindings() -> Vec<PropertyBinding> {
        vec![
            PropertyBinding::new("gloss", ROUGHNESS).unwrap(),
            PropertyBinding::new("tint", "tint").unwrap(),
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
    fn parses_property_and_inverse() {
        assert_eq!(
            "metallic".parse::<ChannelSource>().unwrap(),
            property(METALLIC, false)
        );
        assert_eq!(
            "1-metallic".parse::<ChannelSource>().unwrap(),
            property(METALLIC, true)
        );
    }

    #[test]
    fn parses_inverted_roughness() {
        assert_eq!(
            "roughness".parse::<ChannelSource>().unwrap(),
            property(ROUGHNESS, false)
        );
        assert_eq!(
            "1-roughness".parse::<ChannelSource>().unwrap(),
            property(ROUGHNESS, true)
        );
    }

    #[test]
    fn parses_components_from_either_alias_set() {
        assert_eq!(
            "baseColor.r".parse::<ChannelSource>().unwrap(),
            component(BASE_COLOR, VectorComponent::R, false)
        );
        assert_eq!(
            "1-baseColor.a".parse::<ChannelSource>().unwrap(),
            component(BASE_COLOR, VectorComponent::A, true)
        );
        assert_eq!(
            "normal.x".parse::<ChannelSource>().unwrap(),
            component("normal", VectorComponent::X, false)
        );
    }

    #[test]
    fn keeps_dotted_keys_without_a_component() {
        assert_eq!(
            "my.attr".parse::<ChannelSource>().unwrap(),
            property("my.attr", false)
        );
    }

    #[test]
    fn rejects_empty_property() {
        assert!("".parse::<ChannelSource>().is_err());
        assert!("1-".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!("baseColor.q".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn rejects_a_whitespace_property() {
        // A voxel name with spaces is unreachable here; alias it with
        // --define-property and reference the space-free alias instead.
        assert!("super emissive thing".parse::<ChannelSource>().is_err());
        assert!("1-super emissive thing".parse::<ChannelSource>().is_err());
    }

    #[test]
    fn a_binding_resolves_to_its_concrete_key() {
        // The alias `gloss` renames to the layer's `roughness`.
        let resolved = property("gloss", true).resolve(&bindings()).unwrap();
        assert_eq!(resolved, property(ROUGHNESS, true));

        // A component rides through the rename unchanged; its validity against
        // the value pool kind is checked later, at the bake.
        let resolved = component("tint", VectorComponent::R, false)
            .resolve(&bindings())
            .unwrap();
        assert_eq!(resolved, component("tint", VectorComponent::R, false));
    }

    #[test]
    fn a_binding_reaches_a_name_with_spaces() {
        // The alias is space-free and parses, then resolves to its bound voxel
        // name, spaces and all.
        let bindings = vec![PropertyBinding::new("spark", "super emissive thing").unwrap()];
        let resolved = property("spark", false).resolve(&bindings).unwrap();
        assert_eq!(resolved, property("super emissive thing", false));
    }

    #[test]
    fn resolve_no_longer_validates_the_component() {
        // The type is unknown until the document loads, so resolve accepts both
        // a scalar with a component and a color with none; the `mesh`
        // implementation validates each against its value pool kind.
        assert!(
            component(METALLIC, VectorComponent::R, false)
                .resolve(&bindings())
                .is_ok()
        );
        assert!(
            property_with(BASE_COLOR, None, false)
                .resolve(&bindings())
                .is_ok()
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
