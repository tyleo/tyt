use crate::CliValue;
use voxsmith::{ColorChannel, MaterialChannel, VectorComponent};

/// Parses one `--texture-map` channel expression. `0` and `1` are the
/// constants, `computed-occlusion` the geometry-derived occlusion, a leading
/// `1-` inverts a property, and a trailing `.r`/`.g`/`.b`/`.a` or
/// `.x`/`.y`/`.z`/`.w` reads one component. A property reference carries no
/// whitespace, and its key may be a `--define-property` alias the command
/// resolves before the bake.
pub fn parse_material_channel(text: &str) -> Result<MaterialChannel, String> {
    match text {
        "0" => return Ok(MaterialChannel::Zero),
        "1" => return Ok(MaterialChannel::One),
        "computed-occlusion" => return Ok(MaterialChannel::ComputedOcclusion),
        _ => {}
    }

    let (body, invert) = match text.strip_prefix("1-") {
        Some(rest) => (rest, true),
        None => (text, false),
    };

    // A trailing letter from either alias set selects one component. A
    // longer suffix is part of the name, so dotted keys pass through
    // unsplit.
    let (name, component) = match body.rsplit_once('.') {
        Some((head, tail)) if tail.len() == 1 && tail.chars().all(|c| c.is_ascii_alphabetic()) => {
            (head, Some(color_channel(VectorComponent::parse(tail)?)))
        }
        _ => (body, None),
    };

    if name.is_empty() {
        return Err(format!("`{text}` names no property"));
    }

    if name.chars().any(char::is_whitespace) {
        return Err(format!(
            "`{text}` names a property with whitespace; alias it with --define-property"
        ));
    }

    Ok(MaterialChannel::Property {
        key: name.to_string(),
        component,
        invert,
    })
}

/// The color channel a vector component indexes.
fn color_channel(component: VectorComponent) -> ColorChannel {
    match component.index() {
        0 => ColorChannel::R,
        1 => ColorChannel::G,
        2 => ColorChannel::B,
        _ => ColorChannel::A,
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_material_channel;
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

    #[test]
    fn parses_constants_and_computed_occlusion() {
        assert_eq!(parse_material_channel("0").unwrap(), MaterialChannel::Zero);
        assert_eq!(parse_material_channel("1").unwrap(), MaterialChannel::One);
        assert_eq!(
            parse_material_channel("computed-occlusion").unwrap(),
            MaterialChannel::ComputedOcclusion
        );
    }

    #[test]
    fn parses_property_and_inverse() {
        assert_eq!(
            parse_material_channel("metallic").unwrap(),
            property(METALLIC, false)
        );
        assert_eq!(
            parse_material_channel("1-metallic").unwrap(),
            property(METALLIC, true)
        );
    }

    #[test]
    fn parses_inverted_roughness() {
        assert_eq!(
            parse_material_channel("roughness").unwrap(),
            property(ROUGHNESS, false)
        );
        assert_eq!(
            parse_material_channel("1-roughness").unwrap(),
            property(ROUGHNESS, true)
        );
    }

    #[test]
    fn parses_components_from_either_alias_set() {
        assert_eq!(
            parse_material_channel("baseColor.r").unwrap(),
            component(BASE_COLOR, ColorChannel::R, false)
        );
        assert_eq!(
            parse_material_channel("1-baseColor.a").unwrap(),
            component(BASE_COLOR, ColorChannel::A, true)
        );
        assert_eq!(
            parse_material_channel("normal.x").unwrap(),
            component("normal", ColorChannel::R, false)
        );
        assert_eq!(
            parse_material_channel("normal.w").unwrap(),
            component("normal", ColorChannel::A, false)
        );
    }

    #[test]
    fn keeps_dotted_keys_without_a_component() {
        assert_eq!(
            parse_material_channel("my.attr").unwrap(),
            property("my.attr", false)
        );
    }

    #[test]
    fn rejects_empty_property() {
        assert!(parse_material_channel("").is_err());
        assert!(parse_material_channel("1-").is_err());
    }

    #[test]
    fn rejects_an_unknown_component() {
        assert!(parse_material_channel("baseColor.q").is_err());
    }

    #[test]
    fn rejects_a_whitespace_property() {
        // A voxel name with spaces is unreachable here; alias it with
        // --define-property and reference the space-free alias instead.
        assert!(parse_material_channel("super emissive thing").is_err());
        assert!(parse_material_channel("1-super emissive thing").is_err());
    }
}
