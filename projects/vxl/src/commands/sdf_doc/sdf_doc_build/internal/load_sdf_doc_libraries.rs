use crate::{
    ResolvePrefsPaths, Result,
    commands::{SdfDocConfig, SdfDocLibrariesConfig, SdfDocLibrary, built_in_sdf_doc_config},
};
use std::{
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind},
};
use ty_preferences::{Dependencies as PreferencesDependencies, JsoncCodec, load_application_prefs};

/// The libraries `sdf-doc build` can read, from the built-in layer and each
/// `.vxlconfig` layer's `sdfDoc.build.libraries`. The last layer defining a
/// name in either group wins it.
pub fn load_sdf_doc_libraries(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<BTreeMap<String, SdfDocLibrary>> {
    let paths = dependencies.resolve_prefs_paths()?;

    let layers = load_application_prefs::<SdfDocConfig>(
        dependencies,
        &JsoncCodec,
        &paths,
        ".vxlconfig",
        "sdfDoc",
    )
    .map_err(|error| {
        IOError::new(
            error.kind(),
            format!("a `.vxlconfig` layer failed to load: {error}"),
        )
    })?;

    let mut libraries: BTreeMap<_, _> = built_in_sdf_doc_config()
        .build
        .libraries
        .embedded
        .into_iter()
        .map(|(name, library)| {
            let library = SdfDocLibrary::Embedded {
                document: Box::new(library.document),
                dir: None,
            };
            (name, library)
        })
        .collect();

    for layer in layers {
        let SdfDocLibrariesConfig { embedded, files } = layer.prefs.build.libraries;

        if let Some(name) = files.keys().find(|name| embedded.contains_key(*name)) {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!(
                    "a `.vxlconfig` layer failed to load: `{}` defines the library `{name}` \
                     in both `files` and `embedded`",
                    layer.dir.join(".vxlconfig").display()
                ),
            )
            .into());
        }

        libraries.extend(embedded.into_iter().map(|(name, library)| {
            let library = SdfDocLibrary::Embedded {
                document: Box::new(library.document),
                dir: Some(layer.dir.clone()),
            };
            (name, library)
        }));

        libraries.extend(
            files
                .into_iter()
                .map(|(name, library)| (name, SdfDocLibrary::File(layer.dir.join(library.path)))),
        );
    }

    Ok(libraries)
}

#[cfg(test)]
mod tests {
    use crate::{
        Cascade,
        commands::{SdfDocLibrary, load_sdf_doc_libraries},
    };
    use std::path::{Path, PathBuf};

    /// A `.vxlconfig` holding `libraries` as its `sdfDoc.build.libraries`.
    fn layer(libraries: &str) -> &'static str {
        format!(r#"{{ "sdfDoc": {{ "build": {{ "libraries": {libraries} }} }} }}"#)
            .replace("EMPTY", r#"{ "version": 1 }"#)
            .leak()
    }

    #[test]
    fn a_later_layer_replaces_a_name_in_either_group() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    layer(
                        r#"{
                            "files": { "props": { "path": "kits/props.sdfj" } },
                            "embedded": { "fabrics": { "document": EMPTY } },
                        }"#,
                    ),
                ),
                (
                    "/repo/sub/.vxlconfig",
                    layer(
                        r#"{
                            "files": { "fabrics": { "description": "Cloth", "path": "fabrics.sdfj" } },
                            "embedded": { "props": { "document": EMPTY } },
                        }"#,
                    ),
                ),
            ],
        );

        let libraries = load_sdf_doc_libraries(&cascade).unwrap();

        assert_eq!(
            libraries.keys().collect::<Vec<_>>(),
            ["fabrics", "materials", "props"]
        );
        assert_eq!(
            libraries["fabrics"],
            SdfDocLibrary::File(PathBuf::from("/repo/sub/fabrics.sdfj"))
        );
        assert!(matches!(
            &libraries["props"],
            SdfDocLibrary::Embedded { dir: Some(dir), .. } if dir == Path::new("/repo/sub")
        ));
    }

    #[test]
    fn a_file_path_resolves_against_its_vxlconfig() {
        let cascade = Cascade::new(
            true,
            &[(
                "/repo/.vxlconfig",
                layer(r#"{ "files": { "props": { "path": "kits/props.sdfj" } } }"#),
            )],
        );

        let libraries = load_sdf_doc_libraries(&cascade).unwrap();

        assert_eq!(
            libraries["props"],
            SdfDocLibrary::File(PathBuf::from("/repo/kits/props.sdfj"))
        );
    }

    #[test]
    fn a_layer_defining_a_name_in_both_groups_errors() {
        let cascade = Cascade::new(
            true,
            &[(
                "/repo/.vxlconfig",
                layer(
                    r#"{
                        "files": { "props": { "path": "props.sdfj" } },
                        "embedded": { "props": { "document": EMPTY } },
                    }"#,
                ),
            )],
        );

        let error = load_sdf_doc_libraries(&cascade).unwrap_err().to_string();
        assert_eq!(
            error,
            "a `.vxlconfig` layer failed to load: `/repo/.vxlconfig` defines the library \
             `props` in both `files` and `embedded`"
        );
    }

    #[test]
    fn an_embedded_document_outside_the_sdfj_format_errors() {
        let cascade = Cascade::new(
            true,
            &[(
                "/repo/.vxlconfig",
                layer(r#"{ "embedded": { "props": { "document": { "shapes3d": [] } } } }"#),
            )],
        );

        let error = load_sdf_doc_libraries(&cascade).unwrap_err().to_string();
        assert!(
            error.starts_with("a `.vxlconfig` layer failed to load"),
            "{error}"
        );
    }
}
