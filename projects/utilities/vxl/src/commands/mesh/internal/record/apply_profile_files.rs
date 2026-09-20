use crate::{
    Result,
    commands::{Profile, check_expression, fill_file_template, push_file_write, written_file_name},
};
use voxsmith::operations::mesh::{FileForm, FileWrite, WrittenValue};

/// Fills every file write of `profile`, which `origin` applies, whose
/// destination no flag claimed, each template filled with `file_stem`. A
/// flag's write stands at its destination, so the profile's yields to it.
pub(crate) fn apply_profile_files(
    files: &mut Vec<FileWrite>,
    profile: &Profile,
    origin: &str,
    file_stem: &str,
) -> Result<()> {
    let claimed: Vec<(String, FileForm)> = files
        .iter()
        .map(|write| (write.file.clone(), write.form.clone()))
        .collect();

    for (template, entries) in &profile.files.json {
        let template_origin = format!("{origin}'s files.json template `{template}`");
        let file = written_file_name(&template_origin, &fill_file_template(template, file_stem))?;

        for (name, entry) in entries {
            let form = FileForm::Json { name: name.clone() };

            if claimed.contains(&(file.clone(), form.clone())) {
                continue;
            }

            check_expression(&format!("{template_origin}'s key `{name}`"), &entry.value)?;

            let write = FileWrite {
                file: file.clone(),
                value: WrittenValue {
                    expression: entry.value.clone(),
                    transfer: entry.transfer.0,
                },
                form,
            };

            push_file_write(files, write, origin)?;
        }
    }

    for (template, entry) in &profile.files.png {
        let template_origin = format!("{origin}'s files.png template `{template}`");
        let file = written_file_name(&template_origin, &fill_file_template(template, file_stem))?;

        if claimed.contains(&(file.clone(), FileForm::Png)) {
            continue;
        }

        check_expression(&template_origin, &entry.value)?;

        let write = FileWrite {
            file,
            value: WrittenValue {
                expression: entry.value.clone(),
                transfer: entry.transfer.0,
            },
            form: FileForm::Png,
        };

        push_file_write(files, write, origin)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::apply_profile_files;
    use crate::commands::Profile;
    use voxsmith::operations::mesh::{FileForm, FileWrite, Transfer, WrittenValue};

    fn profile(json: &str) -> Profile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn templates_fill_and_the_flags_writes_stand() {
        let profile = profile(
            r#"{
                "files": {
                    "png": {
                        "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" },
                        "{file-stem}-mse.png": { "transfer": "linear", "value": "mse" }
                    },
                    "json": {
                        "{file-stem}.json": {
                            "ior": { "transfer": "linear", "value": "ior" },
                            "tint": { "transfer": "srgb", "value": "tint" }
                        }
                    }
                }
            }"#,
        );
        let hand = FileWrite {
            file: "lamp-orm.png".to_owned(),
            value: WrittenValue {
                expression: "hand".to_owned(),
                transfer: Transfer::Srgb,
            },
            form: FileForm::Png,
        };
        let mut files = vec![hand.clone()];

        apply_profile_files(&mut files, &profile, "the profile `x`", "lamp").unwrap();

        let names: Vec<_> = files
            .iter()
            .map(|write| (write.file.as_str(), write.value.expression.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("lamp-orm.png", "hand"),
                ("lamp.json", "ior"),
                ("lamp.json", "tint"),
                ("lamp-mse.png", "mse"),
            ]
        );
    }

    #[test]
    fn a_template_filling_to_a_path_errors() {
        let profile = profile(
            r#"{ "files": { "png": { "maps/{file-stem}.png": { "transfer": "linear", "value": "v" } } } }"#,
        );

        let error = apply_profile_files(&mut Vec::new(), &profile, "the profile `x`", "lamp")
            .unwrap_err()
            .to_string();
        assert!(error.contains("`maps/{file-stem}.png`"), "{error}");
    }

    #[test]
    fn two_templates_filling_to_one_file_error() {
        let profile = profile(
            r#"{
                "files": {
                    "png": {
                        "{file-stem}.png": { "transfer": "linear", "value": "a" },
                        "lamp.png": { "transfer": "linear", "value": "b" }
                    }
                }
            }"#,
        );

        assert!(apply_profile_files(&mut Vec::new(), &profile, "the profile `x`", "lamp").is_err());
    }
}
