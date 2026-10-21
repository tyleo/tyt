use crate::{
    Error, ReadFile, ResolvePrefsPaths, Result,
    commands::{SdfDocLibrary, load_sdf_doc_libraries},
};
use sdfj::SdfjFile;
use sdfj_codec::{DependenciesImpl as SdfjCodecDependencies, from_sdfj_or_sdfjz_file_bytes};
use sdfj_sdfcore::from_sdfj_file;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind},
};
use ty_preferences::Dependencies as PreferencesDependencies;

/// One library as the builder reads it.
#[derive(Serialize)]
struct BuilderLibrary<'a> {
    name: &'a str,

    document: SdfjFile,
}

/// The builder's JSON holding each library `names` lists, in list order. Each
/// library passes the sdfj format's checks first. Errors report `origin` as the
/// setting that lists `names`.
pub fn sdfj_builder_libraries(
    dependencies: &(impl PreferencesDependencies + ReadFile + ResolvePrefsPaths),
    origin: &str,
    names: &[String],
) -> Result<Vec<u8>> {
    let defined = if names.is_empty() {
        BTreeMap::new()
    } else {
        load_sdf_doc_libraries(dependencies)?
    };

    let libraries = names
        .iter()
        .map(|name| {
            let document = match defined.get(name) {
                Some(SdfDocLibrary::Embedded(document)) => (**document).clone(),

                Some(SdfDocLibrary::File(path)) => {
                    let failed = |error| {
                        format!(
                            "the library `{name}` failed to read `{}`: {error}",
                            path.display()
                        )
                    };
                    let bytes = ReadFile::read_file(dependencies, path)
                        .map_err(|error| IOError::new(error.kind(), failed(error.to_string())))?;
                    from_sdfj_or_sdfjz_file_bytes(&SdfjCodecDependencies, &bytes).map_err(
                        |error| IOError::new(ErrorKind::InvalidData, failed(error.to_string())),
                    )?
                }

                None => return Err(undefined(origin, name, &defined)),
            };

            from_sdfj_file(&document).map_err(|error| {
                IOError::new(
                    ErrorKind::InvalidData,
                    format!("the library `{name}` breaks the sdfj format: {error}"),
                )
            })?;

            Ok(BuilderLibrary { name, document })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(serde_json::to_vec(&libraries).expect("a checked document writes as JSON"))
}

/// The error for the library `name` that `origin` lists and `defined` lacks.
fn undefined(origin: &str, name: &str, defined: &BTreeMap<String, SdfDocLibrary>) -> Error {
    let names: Vec<_> = defined.keys().map(|name| format!("`{name}`")).collect();

    Error::usage(format!(
        "{origin} asks for the library `{name}`, which is not defined. The libraries are {}",
        names.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::sdfj_builder_libraries};
    use serde_json::Value;

    /// A cascade whose user layer defines file libraries and the embedded
    /// library `velvet`. `missing.sdfj` has no file.
    fn cascade() -> Cascade {
        Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r##"{ "sdfDoc": { "build": { "libraries": {
                        "files": {
                            "broken": { "path": "broken.sdfj" },
                            "missing": { "path": "missing.sdfj" },
                            "props": { "path": "kits/props.sdfj" },
                        },
                        "embedded": { "velvet": { "document": {
                            "version": 1,
                            "materials": [{ "kind": "material", "properties": { "baseColor": "#5A1030" } }],
                            "names": { "materials": { "velvet": 0 } },
                        } } },
                    } } } }"##,
                ),
                ("/home/kits/props.sdfj", PROPS),
                ("/home/broken.sdfj", BROKEN),
            ],
        )
    }

    /// A library naming the material `crate`.
    const PROPS: &str = r##"{
        "version": 1,
        "materials": [{ "kind": "material", "properties": { "baseColor": "#8A5A2B" } }],
        "names": { "materials": { "crate": 0 } }
    }"##;

    /// A library naming a material past the end of its table.
    const BROKEN: &str = r#"{ "version": 1, "names": { "materials": { "crate": 5 } } }"#;

    /// The libraries JSON for `names`, or the error message.
    fn libraries(names: &[&str]) -> Result<Value, String> {
        let names: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();

        sdfj_builder_libraries(&cascade(), "--library", &names)
            .map(|bytes| serde_json::from_slice(&bytes).unwrap())
            .map_err(|error| error.to_string())
    }

    #[test]
    fn no_names_read_no_library() {
        let unreadable = Cascade::new(true, &[("/home/.vxlconfig", "{ not jsonc")]);

        let bytes = sdfj_builder_libraries(&unreadable, "--library", &[]).unwrap();

        assert_eq!(bytes, b"[]");
    }

    #[test]
    fn each_library_writes_its_name_and_document_in_list_order() {
        let libraries = libraries(&["props", "velvet", "materials"]).unwrap();

        let named = |index: usize, table: &str| {
            let library = &libraries[index];
            let names = library["document"]["names"][table].as_object().unwrap();
            (
                library["name"].as_str().unwrap().to_owned(),
                names.keys().next().unwrap().clone(),
            )
        };

        assert_eq!(libraries.as_array().unwrap().len(), 3);
        assert_eq!(
            named(0, "materials"),
            ("props".to_owned(), "crate".to_owned())
        );
        assert_eq!(
            named(1, "materials"),
            ("velvet".to_owned(), "velvet".to_owned())
        );
        assert_eq!(
            named(2, "materials"),
            ("materials".to_owned(), "amethyst".to_owned())
        );
    }

    #[test]
    fn an_undefined_library_errors_with_the_defined_ones() {
        assert_eq!(
            libraries(&["fabrics"]).unwrap_err(),
            "error: --library asks for the library `fabrics`, which is not defined. The \
             libraries are `broken`, `materials`, `missing`, `props`, `velvet`\n"
        );
    }

    #[test]
    fn a_library_file_that_fails_to_read_or_check_errors() {
        let missing = libraries(&["missing"]).unwrap_err();
        assert!(
            missing.starts_with("the library `missing` failed to read `/home/missing.sdfj`"),
            "{missing}"
        );

        assert_eq!(
            libraries(&["broken"]).unwrap_err(),
            "the library `broken` breaks the sdfj format: names.materials gives `crate` to \
             materials[5], past the end of materials"
        );
    }
}
