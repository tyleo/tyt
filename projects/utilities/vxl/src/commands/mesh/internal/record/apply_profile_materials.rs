use crate::{
    Result,
    commands::{
        MaterialTable, Profile, SlotEntry, check_expression, extra_write, fill_file_template,
        parse_uv_list,
    },
};
use voxsmith::operations::mesh::{SlotSource, SlotWrite};

/// Fills every material element of `profile`, which `origin` applies, whose
/// destination no flag claimed. A flag's element stands at its destination,
/// so the profile's yields to it.
pub(crate) fn apply_profile_materials(
    materials: &mut MaterialTable,
    profile: &Profile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    for (index, entry) in (0..).zip(&profile.materials) {
        let entry_origin = format!("{origin}'s materials entry {index}");
        let material = materials.material(&entry_origin, index)?;

        if material.name.is_none() {
            material.name = entry.name.clone();
        }

        if material.uv_streams.is_none()
            && let Some(uvs) = &entry.uvs
        {
            material.uv_streams = Some(parse_uv_list(&entry_origin, uvs)?);
        }

        for (property, slot) in &entry.slots {
            if material
                .slots
                .iter()
                .any(|existing| &existing.property == property)
            {
                continue;
            }

            let source = match slot {
                SlotEntry::File { file } => SlotSource::File(fill_file_template(file, file_stem)),

                SlotEntry::Value { value } => {
                    check_expression(&format!("{entry_origin}'s slot `{property}`"), value)?;
                    SlotSource::Value(value.clone())
                }
            };

            material.slots.push(SlotWrite {
                property: property.clone(),
                source,
            });
        }

        for (name, extra) in &entry.extras {
            if material
                .extras
                .iter()
                .any(|existing| &existing.name == name)
            {
                continue;
            }

            material.extras.push(extra_write(
                &format!("{entry_origin}'s extra `{name}`"),
                name,
                extra,
                file_stem,
            )?);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_profile_materials;
    use crate::commands::{MaterialTable, Profile};
    use voxsmith::operations::mesh::{ArrayDomain, SlotSource, SlotWrite};

    fn profile() -> Profile {
        serde_json::from_str(
            r#"{
                "materials": [
                    {
                        "name": "body",
                        "uvs": ["swatch", "face"],
                        "slots": {
                            "baseColorTexture": { "kind": "value", "value": "albedo" },
                            "occlusionTexture": { "kind": "file", "file": "{file-stem}-ao.png" }
                        }
                    },
                    { "name": "glow" }
                ]
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn the_flags_elements_stand_and_the_rest_fills() {
        let mut materials = MaterialTable::declared(2, "the profile `x`".to_owned());
        let hand = materials.material("--material-name", 0).unwrap();
        hand.name = Some("hand".to_owned());
        hand.uv_streams = Some(vec![ArrayDomain::Voxel]);
        hand.slots.push(SlotWrite {
            property: "baseColorTexture".to_owned(),
            source: SlotSource::Value("hand".to_owned()),
        });

        apply_profile_materials(&mut materials, &profile(), "the profile `x`", "lamp").unwrap();

        let materials = materials.finish().unwrap();
        let [body, glow] = materials.as_slice() else {
            panic!("two materials");
        };
        assert_eq!(body.name.as_deref(), Some("hand"));
        assert_eq!(body.uv_streams, Some(vec![ArrayDomain::Voxel]));
        assert_eq!(
            body.slots,
            [
                SlotWrite {
                    property: "baseColorTexture".to_owned(),
                    source: SlotSource::Value("hand".to_owned()),
                },
                SlotWrite {
                    property: "occlusionTexture".to_owned(),
                    source: SlotSource::File("lamp-ao.png".to_owned()),
                },
            ]
        );
        assert_eq!(glow.name.as_deref(), Some("glow"));
    }

    #[test]
    fn a_material_past_the_flags_count_errors() {
        let mut materials = MaterialTable::declared(1, "--material-count 1".to_owned());

        let error = apply_profile_materials(&mut materials, &profile(), "the profile `x`", "lamp")
            .unwrap_err()
            .to_string();
        assert!(error.contains("materials entry 1"), "{error}");
        assert!(error.contains("--material-count 1"), "{error}");
    }
}
