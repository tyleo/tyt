use crate::{
    Result,
    commands::{Profile, extra_write},
};
use voxsmith::operations::mesh::ExtraWrite;

/// Fills every mesh extra of `profile`, which `origin` applies, whose name no
/// flag claimed. A flag's extra stands under its name, so the profile's
/// yields to it.
pub(crate) fn apply_profile_mesh_extras(
    extras: &mut Vec<ExtraWrite>,
    profile: &Profile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    for (name, entry) in &profile.mesh_extras {
        if extras.iter().any(|existing| &existing.name == name) {
            continue;
        }

        extras.push(extra_write(
            &format!("{origin}'s meshExtras entry `{name}`"),
            name,
            entry,
            file_stem,
        )?);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_profile_mesh_extras;
    use crate::commands::Profile;
    use voxsmith::operations::mesh::{ExtraForm, ExtraSource, ExtraWrite};

    #[test]
    fn the_flags_extra_stands_and_the_rest_fills() {
        let profile: Profile = serde_json::from_str(
            r#"{
                "meshExtras": {
                    "albedo": { "kind": "json-value", "value": "albedo", "transfer": "linear" },
                    "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" }
                }
            }"#,
        )
        .unwrap();
        let hand = ExtraWrite {
            name: "albedo".to_owned(),
            form: ExtraForm::Json,
            source: ExtraSource::File("albedo.json".to_owned()),
        };
        let mut extras = vec![hand.clone()];

        apply_profile_mesh_extras(&mut extras, &profile, "the profile `x`", "lamp").unwrap();

        assert_eq!(extras[0], hand);
        assert_eq!(extras[1].name, "heat");
        assert_eq!(
            extras[1].source,
            ExtraSource::File("lamp-heat.png".to_owned())
        );
    }
}
