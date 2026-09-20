use crate::{
    ResolvePrefsPaths, Result,
    commands::{MeshConfig, ProfileSet},
};
use std::io::Error as IOError;
use ty_preferences::{Dependencies as PreferencesDependencies, JsoncCodec, load_application_prefs};

/// The profiles a run can apply: the built-ins under the `.vxlconfig` layers,
/// the user's first and the working directory's last.
pub(crate) fn load_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet> {
    let paths = dependencies.resolve_prefs_paths()?;

    let layers = load_application_prefs::<MeshConfig>(
        dependencies,
        &JsoncCodec,
        &paths,
        ".vxlconfig",
        "mesh",
    )
    .map_err(|error| {
        IOError::new(
            error.kind(),
            format!("a `.vxlconfig` layer failed to load: {error}"),
        )
    })?;

    Ok(ProfileSet::layered(
        layers.into_iter().map(|layer| layer.prefs.profiles),
    ))
}

#[cfg(test)]
mod tests {
    use super::load_profile_set;
    use crate::{ResolvePrefsPaths, commands::ProfileSet};
    use std::{
        collections::BTreeMap,
        io::Result as IOResult,
        path::{Path, PathBuf},
    };
    use ty_preferences::{Dependencies as PreferencesDependencies, PrefsPaths};

    /// A cascade over in-memory files, the working directory `/repo/sub` under
    /// the git root `/repo` and the user's home `/home`.
    struct Cascade {
        in_repository: bool,
        files: BTreeMap<PathBuf, &'static str>,
    }

    impl Cascade {
        fn new(in_repository: bool, files: &[(&str, &'static str)]) -> Self {
            Cascade {
                in_repository,
                files: files
                    .iter()
                    .map(|&(path, text)| (PathBuf::from(path), text))
                    .collect(),
            }
        }
    }

    impl PreferencesDependencies for Cascade {
        fn read_file(&self, path: &Path) -> IOResult<Option<Vec<u8>>> {
            Ok(self.files.get(path).map(|text| text.as_bytes().to_vec()))
        }

        fn write_file(&self, _: &Path, _: &[u8]) -> IOResult<()> {
            unreachable!("loading never writes")
        }
    }

    impl ResolvePrefsPaths for Cascade {
        fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths> {
            Ok(PrefsPaths {
                cwd: PathBuf::from("/repo/sub"),
                git_root: self.in_repository.then(|| PathBuf::from("/repo")),
                user: Some(PathBuf::from("/home")),
            })
        }
    }

    /// The values of the profile `name`.
    fn values(profiles: &ProfileSet, name: &str) -> Vec<String> {
        profiles.get("the test", name).unwrap().values.clone()
    }

    #[test]
    fn the_layers_load_in_application_order_over_the_built_ins() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "mesh": { "profiles": {
                        "a": { "values": ["a = 1"] },
                        "orm": { "values": ["orm = 1"] },
                    } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "mesh": { "profiles": { "a": { "values": ["a = 2"] } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "mesh": { "profiles": { "b": { "values": ["b = a"] } } } }"#,
                ),
            ],
        );

        let profiles = load_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 2"]);
        assert_eq!(values(&profiles, "b"), ["b = a"]);
        assert_eq!(values(&profiles, "orm"), ["orm = 1"]);
        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn outside_a_repository_the_user_layer_alone_loads() {
        let cascade = Cascade::new(
            false,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "mesh": { "profiles": { "a": { "values": ["a = 1"] } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "mesh": { "profiles": { "b": { "values": ["b = a"] } } } }"#,
                ),
            ],
        );

        let profiles = load_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 1"]);
        assert!(profiles.get("the test", "b").is_err());
    }

    #[test]
    fn a_layer_without_the_section_supplies_nothing() {
        let cascade = Cascade::new(true, &[("/repo/.vxlconfig", r#"{ "fs": {} }"#)]);

        let profiles = load_profile_set(&cascade).unwrap();

        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn a_broken_layer_errors_naming_the_file() {
        for text in [
            r#"{ "mesh": { "profiles": { "a": { "slot": 1 } } } }"#,
            r#"{ "mesh": { "profiles": { "a": } } }"#,
        ] {
            let cascade = Cascade::new(true, &[("/repo/.vxlconfig", text)]);

            let error = load_profile_set(&cascade).unwrap_err().to_string();
            assert!(error.contains("`.vxlconfig`"), "{error}");
        }
    }
}
