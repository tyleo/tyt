use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{MeshProfile, ObjectConfig, built_in_profiles},
    load_profile_set,
};
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `object mesh` can apply: the built-ins under the `.vxlconfig`
/// layers' `object.mesh.profiles`, the user's first and the working
/// directory's last.
pub fn load_mesh_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<MeshProfile>> {
    load_profile_set(
        dependencies,
        "object",
        built_in_profiles(),
        |config: ObjectConfig| config.mesh.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        Cascade, ProfileSet,
        commands::{MeshProfile, load_mesh_profile_set},
    };

    /// The values of the profile `name`.
    fn values(profiles: &ProfileSet<MeshProfile>, name: &str) -> Vec<String> {
        profiles.get("the test", name).unwrap().values.clone()
    }

    #[test]
    fn the_layers_load_in_application_order_over_the_built_ins() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": {
                        "a": { "values": ["a = 1"] },
                        "orm": { "values": ["orm = 1"] },
                    } } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "a": { "values": ["a = 2"] } } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "b": { "values": ["b = a"] } } } } }"#,
                ),
            ],
        );

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 2"]);
        assert_eq!(values(&profiles, "b"), ["b = a"]);
        assert_eq!(values(&profiles, "orm"), ["orm = 1"]);
        assert!(profiles.get("the test", "pbr").is_ok());

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, profiles)| {
                let names: Vec<_> = profiles.into_iter().map(|(name, _)| name).collect();

                format!("{origin}: {}", names.join(" "))
            })
            .collect();
        assert_eq!(
            groups,
            [
                "built in: albedo defaults emissive glass pbr",
                "/home/.vxlconfig: orm",
                "/repo/.vxlconfig: a",
                "/repo/sub/.vxlconfig: b",
            ]
        );
    }

    #[test]
    fn outside_a_repository_the_user_layer_alone_loads() {
        let cascade = Cascade::new(
            false,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "a": { "values": ["a = 1"] } } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "object": { "mesh": { "profiles": { "b": { "values": ["b = a"] } } } } }"#,
                ),
            ],
        );

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert_eq!(values(&profiles, "a"), ["a = 1"]);
        assert!(profiles.get("the test", "b").is_err());
    }

    #[test]
    fn a_layer_without_the_section_supplies_nothing() {
        let cascade = Cascade::new(true, &[("/repo/.vxlconfig", r#"{ "fs": {} }"#)]);

        let profiles = load_mesh_profile_set(&cascade).unwrap();

        assert!(profiles.get("the test", "pbr").is_ok());
    }

    #[test]
    fn a_broken_layer_errors_naming_the_file() {
        for text in [
            r#"{ "object": { "mesh": { "profiles": { "a": { "slot": 1 } } } } }"#,
            r#"{ "object": { "mesh": { "profiles": { "a": } } } }"#,
        ] {
            let cascade = Cascade::new(true, &[("/repo/.vxlconfig", text)]);

            let error = load_mesh_profile_set(&cascade).unwrap_err().to_string();
            assert!(error.contains("`.vxlconfig`"), "{error}");
        }
    }
}
