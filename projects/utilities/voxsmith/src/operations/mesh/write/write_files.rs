use crate::{
    Error, Result,
    dependencies::mesh::EncodePng,
    operations::mesh::{
        Atlases, FileForm, MeshElement, MeshRecord, ProgramRun, Streams, bake_png, json_text,
    },
};
use meshdoc::MeshFile;
use vox_value_language::eval_expression;

/// A file as it gathers. A JSON object takes one entry per write.
enum PendingFile {
    Json(Vec<(String, String)>),
    Png(Vec<u8>),
}

/// The files `record` writes in first-written order: each png baked on its
/// atlas, and each JSON path's entries merged into one object.
pub(crate) fn write_files<D: EncodePng>(
    dependencies: &D,
    record: &MeshRecord,
    run: &ProgramRun,
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
