use crate::{
    Result,
    commands::{ExtraEntry, check_expression, fill_file_template},
};
use voxsmith::operations::mesh::{ExtraForm, ExtraSource, ExtraWrite, Transfer, WrittenValue};

/// The extras write `entry` describes under `name`, which `origin` holds, its
/// file template filled with `file_stem`.
pub(crate) fn extra_write(
    origin: &str,
    name: &str,
    entry: &ExtraEntry,
    file_stem: &str,
) -> Result<ExtraWrite> {
    let (form, source) = match entry {
        ExtraEntry::ImageFile { file } => (
            ExtraForm::Image,
            ExtraSource::File(fill_file_template(file, file_stem)),
        ),

        ExtraEntry::ImageValue { transfer, value } => {
            (ExtraForm::Image, value_source(origin, value, transfer.0)?)
        }

        ExtraEntry::JsonFile { file } => (
            ExtraForm::Json,
            ExtraSource::File(fill_file_template(file, file_stem)),
        ),

        ExtraEntry::JsonValue { transfer, value } => {
            (ExtraForm::Json, value_source(origin, value, transfer.0)?)
        }
    };

    Ok(ExtraWrite {
        name: name.to_owned(),
        form,
        source,
    })
}

/// A written value source holding `value`, checked to parse.
fn value_source(origin: &str, value: &str, transfer: Transfer) -> Result<ExtraSource> {
    check_expression(origin, value)?;

    Ok(ExtraSource::Value(WrittenValue {
        expression: value.to_owned(),
        transfer,
    }))
}

#[cfg(test)]
mod tests {
    use super::extra_write;
    use crate::commands::ExtraEntry;
    use voxsmith::operations::mesh::{ExtraForm, ExtraSource, Transfer};

    #[test]
    fn each_kind_lowers_to_its_form_and_source() {
        let write = extra_write(
            "the profile `x`",
            "heat",
            &serde_json::from_str::<ExtraEntry>(
                r#"{ "kind": "image-file", "file": "{file-stem}-heat.png" }"#,
            )
            .unwrap(),
            "lamp",
        )
        .unwrap();
        assert_eq!(write.form, ExtraForm::Image);
        assert_eq!(write.source, ExtraSource::File("lamp-heat.png".to_owned()));

        let write = extra_write(
            "the profile `x`",
            "accent",
            &serde_json::from_str::<ExtraEntry>(
                r#"{ "kind": "json-value", "value": "avg(baseColorFactor.rgb)", "transfer": "srgb" }"#,
            )
            .unwrap(),
            "lamp",
        )
        .unwrap();
        assert_eq!(write.form, ExtraForm::Json);
        let ExtraSource::Value(value) = write.source else {
            panic!("a value source");
        };
        assert_eq!(value.transfer, Transfer::Srgb);

        assert!(
            extra_write(
                "the profile `x`",
                "accent",
                &serde_json::from_str::<ExtraEntry>(
                    r#"{ "kind": "json-value", "value": "1 +", "transfer": "srgb" }"#,
                )
                .unwrap(),
                "lamp",
            )
            .is_err()
        );
    }
}
