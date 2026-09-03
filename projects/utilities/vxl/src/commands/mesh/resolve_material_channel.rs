use crate::{
    Result,
    commands::{PropertyBinding, computed_occlusion_unsupported},
};
use voxsmith::MaterialChannel;

/// Resolves a parsed channel's property key against the `--define-property`
/// bindings and rejects `computed-occlusion` under the palette atlas. The
/// color component is checked later, once the document loads, against the
/// key's value pool kind, since the type lives in the file, not the flags.
pub(crate) fn resolve_material_channel(
    channel: &MaterialChannel,
    bindings: &[PropertyBinding],
) -> Result<MaterialChannel> {
    let MaterialChannel::Property {
        key,
        component,
        invert,
    } = channel
    else {
        if let MaterialChannel::ComputedOcclusion = channel {
            return Err(computed_occlusion_unsupported());
        }

        return Ok(channel.clone());
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

    Ok(MaterialChannel::Property {
        key: resolved_key,
        component: *component,
        invert: *invert,
    })
}

#[cfg(test)]
mod tests {
    use crate::commands::{PropertyBinding, resolve_material_channel};
    use voxsmith::{
        ColorChannel, MaterialChannel,
        voxcore::material::{BASE_COLOR, METALLIC, ROUGHNESS},
    };

    fn property(key: &str, invert: bool) -> MaterialChannel {
        MaterialChannel::Property {
            key: key.to_string(),
            component: None,
            invert,
        }
    }

    fn component(key: &str, component: ColorChannel, invert: bool) -> MaterialChannel {
        MaterialChannel::Property {
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
    fn a_binding_resolves_to_its_concrete_key() {
        // The alias `gloss` renames to the layer's `roughness`.
        let resolved = resolve_material_channel(&property("gloss", true), &bindings()).unwrap();
        assert_eq!(resolved, property(ROUGHNESS, true));

        // A component rides through the rename unchanged; its validity against
        // the value pool kind is checked later, at the bake.
        let resolved =
            resolve_material_channel(&component("tint", ColorChannel::R, false), &bindings())
                .unwrap();
        assert_eq!(resolved, component("tint", ColorChannel::R, false));
    }

    #[test]
    fn a_binding_reaches_a_name_with_spaces() {
        // The alias is space-free and parses, then resolves to its bound voxel
        // name, spaces and all.
        let bindings = vec![PropertyBinding::new("spark", "super emissive thing").unwrap()];
        let resolved = resolve_material_channel(&property("spark", false), &bindings).unwrap();
        assert_eq!(resolved, property("super emissive thing", false));
    }

    #[test]
    fn resolve_does_not_check_the_component() {
        // The type is unknown until the document loads, so resolve accepts both
        // a scalar with a component and a color with none; the bake checks
        // each against its value pool kind.
        assert!(
            resolve_material_channel(&component(METALLIC, ColorChannel::R, false), &bindings())
                .is_ok()
        );
        assert!(resolve_material_channel(&property(BASE_COLOR, false), &bindings()).is_ok());
    }

    #[test]
    fn constants_pass_and_computed_occlusion_is_rejected() {
        assert_eq!(
            resolve_material_channel(&MaterialChannel::One, &bindings()).unwrap(),
            MaterialChannel::One
        );
        assert!(
            resolve_material_channel(&MaterialChannel::ComputedOcclusion, &bindings()).is_err()
        );
    }
}
