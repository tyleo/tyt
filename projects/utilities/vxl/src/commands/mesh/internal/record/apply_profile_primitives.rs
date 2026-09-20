use crate::{
    Error, Result,
    commands::{PrimitiveTable, Profile, check_expression, parse_uv_list},
};
use voxsmith::operations::mesh::{AttributeWrite, WrittenValue};

/// Fills every per-primitive element of `profile`, which `origin` applies,
/// whose destination no flag claimed. A flag's element stands at its
/// destination, so the profile's yields to it.
pub(crate) fn apply_profile_primitives(
    primitives: &mut PrimitiveTable,
    profile: &Profile,
    origin: &str,
) -> Result<()> {
    for (index, entry) in (0..).zip(&profile.primitives) {
        let entry_origin = format!("{origin}'s primitives entry {index}");

        if let Some(normal) = entry.normal
            && !primitives.has_normal(index)
        {
            primitives.set_normal(&entry_origin, index, normal)?;
        }

        let primitive = primitives.primitive(&entry_origin, index)?;

        if primitive.name.is_none() {
            primitive.name = entry.name.clone();
        }

        if primitive.uv_streams.is_none()
            && let Some(uvs) = &entry.uvs
        {
            primitive.uv_streams = Some(parse_uv_list(&entry_origin, uvs)?);
        }

        for (attribute, expression) in &entry.builtins {
            if attribute.starts_with('_') {
                return Err(Error::usage(format!(
                    "{entry_origin}'s builtins key `{attribute}` is custom, so it goes under \
                     customs"
                )));
            }

            if primitive
                .attributes
                .iter()
                .any(|write| write.name() == attribute)
            {
                continue;
            }

            check_expression(
                &format!("{entry_origin}'s builtins key `{attribute}`"),
                expression,
            )?;

            primitive.attributes.push(AttributeWrite::Builtin {
                attribute: attribute.clone(),
                expression: expression.clone(),
            });
        }

        for (name, value) in &entry.customs {
            if !name.starts_with('_') {
                return Err(Error::usage(format!(
                    "{entry_origin}'s customs key `{name}` lacks glTF's leading underscore, as \
                     `_{name}`"
                )));
            }

            if primitive
                .attributes
                .iter()
                .any(|write| write.name() == name)
            {
                continue;
            }

            check_expression(
                &format!("{entry_origin}'s customs key `{name}`"),
                &value.value,
            )?;

            primitive.attributes.push(AttributeWrite::Custom {
                name: name.clone(),
                value: WrittenValue {
                    expression: value.value.clone(),
                    transfer: value.transfer.0,
                },
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_profile_primitives;
    use crate::commands::{PrimitiveTable, Profile};
    use voxsmith::operations::mesh::{ArrayDomain, AttributeWrite};

    fn profile(json: &str) -> Profile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn the_flags_elements_stand_and_the_rest_fills() {
        let profile = profile(
            r#"{
                "primitives": [
                    {
                        "name": "body",
                        "normal": false,
                        "uvs": ["face"],
                        "builtins": { "COLOR_0": "albedo" },
                        "customs": { "_HEAT": { "value": "heat", "transfer": "linear" } }
                    }
                ]
            }"#,
        );
        let mut primitives = PrimitiveTable::new(Vec::new(), 0);
        primitives
            .set_normal("--write-primitive-normal", 0, true)
            .unwrap();
        primitives
            .primitive("--write-primitive-uv", 0)
            .unwrap()
            .uv_streams = Some(vec![ArrayDomain::Swatch]);
        primitives
            .primitive("--write-primitive-builtin-value", 0)
            .unwrap()
            .attributes
            .push(AttributeWrite::Builtin {
                attribute: "COLOR_0".to_owned(),
                expression: "hand".to_owned(),
            });

        apply_profile_primitives(&mut primitives, &profile, "the profile `x`").unwrap();

        let primitives = primitives.finish();
        let [primitive] = primitives.as_slice() else {
            panic!("one primitive");
        };
        assert_eq!(primitive.name.as_deref(), Some("body"));
        assert!(primitive.normal);
        assert_eq!(primitive.uv_streams, Some(vec![ArrayDomain::Swatch]));
        assert_eq!(primitive.attributes.len(), 2);
        assert_eq!(
            primitive.attributes[0],
            AttributeWrite::Builtin {
                attribute: "COLOR_0".to_owned(),
                expression: "hand".to_owned(),
            }
        );
        assert_eq!(primitive.attributes[1].name(), "_HEAT");
    }

    #[test]
    fn the_attribute_keys_enforce_the_underscore_rule() {
        let mut primitives = PrimitiveTable::new(Vec::new(), 0);

        assert!(
            apply_profile_primitives(
                &mut primitives,
                &profile(r#"{ "primitives": [{ "builtins": { "_HEAT": "heat" } }] }"#),
                "the profile `x`",
            )
            .is_err()
        );
        assert!(
            apply_profile_primitives(
                &mut primitives,
                &profile(
                    r#"{ "primitives": [{ "customs": { "HEAT": { "value": "heat", "transfer": "linear" } } }] }"#
                ),
                "the profile `x`",
            )
            .is_err()
        );
    }
}
