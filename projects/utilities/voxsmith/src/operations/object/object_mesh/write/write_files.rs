use crate::{
    Error, Result,
    dependencies::object::EncodePng,
    operations::object::{
        Atlases, FileForm, MeshElement, MeshRecord, ProgramRun, Streams, Transfer, bake_png,
        encode_components,
    },
};
use meshdoc::MeshFile;
use serde_json::Value as JsonValue;
use vox_value_language::{Components, Scalar, Value, eval_expression};

/// A file as it gathers. A JSON object takes one entry per write.
enum PendingFile {
    Json(Vec<(String, String)>),

    Png(Vec<u8>),
}

/// The files `record` writes in first-written order: each png baked on its
/// atlas, and each JSON path's entries merged into one object.
pub fn write_files<D: EncodePng>(
    dependencies: &D,
    record: &MeshRecord,
    run: &ProgramRun<'_>,
    streams: &Streams,
    atlases: &Atlases<'_>,
) -> Result<Vec<MeshFile>> {
    let mut files: Vec<(String, PendingFile)> = Vec::new();

    let destinations = run
        .destinations
        .iter()
        .filter(|checked| matches!(checked.destination.element, MeshElement::File { .. }));

    for (file, checked) in record.files.iter().zip(destinations) {
        let element = &checked.destination.element;

        assert_eq!(
            *element,
            MeshElement::File {
                file: file.file.clone()
            },
            "the file destinations stand in record order"
        );

        let value = eval_expression(&checked.expression, &run.evaluated)
            .map_err(|error| Error::mesh_record(element.clone(), error))?;

        let existing = files
            .iter_mut()
            .find(|(name, _)| *name == file.file)
            .map(|(_, pending)| pending);

        match (&file.form, existing) {
            (FileForm::Json { name }, None) => {
                let text = json_text(element, &value, file.value.transfer)?;

                files.push((
                    file.file.clone(),
                    PendingFile::Json(vec![(name.clone(), text)]),
                ));
            }

            (FileForm::Json { name }, Some(PendingFile::Json(entries))) => {
                if entries.iter().any(|(written, _)| written == name) {
                    return Err(Error::mesh_record(
                        element.clone(),
                        format!("writes `{name}` twice"),
                    ));
                }

                let text = json_text(element, &value, file.value.transfer)?;

                entries.push((name.clone(), text));
            }

            (FileForm::Json { .. }, Some(PendingFile::Png(_)))
            | (FileForm::Png, Some(PendingFile::Json(_))) => {
                return Err(Error::mesh_record(
                    element.clone(),
                    "is written as both a png and a JSON object",
                ));
            }

            (FileForm::Png, None) => {
                let image = bake_png(
                    element,
                    &value,
                    file.value.transfer,
                    streams.bake(element),
                    atlases,
                )?;

                let bytes = dependencies.encode_png(&image).map_err(Error::Png)?;

                files.push((file.file.clone(), PendingFile::Png(bytes)));
            }

            (FileForm::Png, Some(PendingFile::Png(_))) => {
                return Err(Error::mesh_record(element.clone(), "is written twice"));
            }
        }
    }

    Ok(files
        .into_iter()
        .map(|(name, pending)| MeshFile {
            name,
            bytes: match pending {
                PendingFile::Json(entries) => json_object(&entries).into_bytes(),
                PendingFile::Png(bytes) => bytes,
            },
        })
        .collect())
}

/// The text of one object holding `entries`, a key per line.
fn json_object(entries: &[(String, String)]) -> String {
    let lines: Vec<String> = entries
        .iter()
        .map(|(name, text)| {
            let key = serde_json::to_string(name).expect("a key serializes");
            format!("  {key}: {text}")
        })
        .collect();

    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

/// The JSON text of `value` under `transfer`, indented for a key of a file's
/// object. An entry of two or more components writes as a row, and an array
/// writes one row per entry.
fn json_text(element: &MeshElement, value: &Value, transfer: Transfer) -> Result<String> {
    if transfer == Transfer::Srgb && value.scalar() != Scalar::F64 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {}, and `srgb` transfers f64 alone", value.to_type()),
        ));
    }

    let width = value.dimension().width();

    let entries: Vec<String> = match value.components() {
        Components::Bool(components) => rows(components, width, |flag| flag.to_string()),

        Components::F64(components) => components
            .chunks_exact(width)
            .map(|entry| {
                let encoded = encode_components(element, entry, transfer, false)?;

                Ok(row(encoded
                    .iter()
                    .map(|&component| {
                        serde_json::to_string(&component).expect("a finite float serializes")
                    })
                    .collect()))
            })
            .collect::<Result<_>>()?,

        Components::String(components) => rows(components, width, |text| {
            JsonValue::String(text.clone()).to_string()
        }),

        Components::U8(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),

        Components::U16(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),

        Components::U32(components) => rows(components, width, |number| {
            JsonValue::from(*number).to_string()
        }),
    };

    Ok(if value.domain().is_array() {
        if entries.is_empty() {
            "[]".to_owned()
        } else {
            format!("[\n    {}\n  ]", entries.join(",\n    "))
        }
    } else {
        entries
            .into_iter()
            .next()
            .expect("a plain value holds one entry")
    })
}

/// Each entry of `components` as a row of `width` leaves.
fn rows<T>(components: &[T], width: usize, leaf: impl Fn(&T) -> String) -> Vec<String> {
    components
        .chunks_exact(width)
        .map(|entry| row(entry.iter().map(&leaf).collect()))
        .collect()
}

/// One entry's text, bare for a lone leaf and bracketed for more.
fn row(leaves: Vec<String>) -> String {
    match leaves.as_slice() {
        [leaf] => leaf.clone(),
        _ => format!("[{}]", leaves.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        MeshElement, Transfer, object_mesh::write::write_files::json_text,
    };
    use vox_value_language::{Components, Dimension, Domain, Value};

    fn element() -> MeshElement {
        MeshElement::File {
            file: "bar.json".to_owned(),
        }
    }

    fn text(value: Value, transfer: Transfer) -> String {
        json_text(&element(), &value, transfer).unwrap()
    }

    #[test]
    fn a_plain_value_writes_bare_and_an_array_writes_rows() {
        let plain = Value::new(Domain::Plain, Dimension::Vec1, Components::F64(vec![0.4])).unwrap();
        assert_eq!(text(plain, Transfer::Linear), "0.4");

        let third = Value::new(
            Domain::Plain,
            Dimension::Vec1,
            Components::F64(vec![1.0 / 3.0]),
        )
        .unwrap();
        assert_eq!(text(third, Transfer::Linear), "0.3333333333333333");

        let color = Value::new(
            Domain::Plain,
            Dimension::Vec4,
            Components::F64(vec![1.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();
        assert_eq!(text(color, Transfer::Linear), "[1.0, 0.0, 0.0, 1.0]");

        let swatches = Value::new(
            Domain::Swatch,
            Dimension::Vec2,
            Components::U8(vec![1, 2, 3, 4]),
        )
        .unwrap();
        assert_eq!(
            text(swatches, Transfer::Linear),
            "[\n    [1, 2],\n    [3, 4]\n  ]"
        );

        let flags = Value::new(
            Domain::Face,
            Dimension::Vec1,
            Components::Bool(vec![true, false]),
        )
        .unwrap();
        assert_eq!(
            text(flags, Transfer::Linear),
            "[\n    true,\n    false\n  ]"
        );

        let names = Value::new(
            Domain::Plain,
            Dimension::Vec1,
            Components::String(vec!["a \"b\"".to_owned()]),
        )
        .unwrap();
        assert_eq!(text(names, Transfer::Linear), "\"a \\\"b\\\"\"");

        let none = Value::new(Domain::Face, Dimension::Vec1, Components::U32(vec![])).unwrap();
        assert_eq!(text(none, Transfer::Linear), "[]");
    }

    #[test]
    fn srgb_curves_the_floats_and_refuses_every_other_scalar() {
        let grey = Value::new(Domain::Plain, Dimension::Vec1, Components::F64(vec![0.5])).unwrap();
        let curved: f64 = text(grey, Transfer::Srgb).parse().unwrap();
        assert!((curved - 0.7354).abs() < 1e-4);

        let flag =
            Value::new(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])).unwrap();
        assert!(json_text(&element(), &flag, Transfer::Srgb).is_err());

        let count = Value::new(Domain::Plain, Dimension::Vec1, Components::U32(vec![1])).unwrap();
        assert!(json_text(&element(), &count, Transfer::Srgb).is_err());
    }
}
