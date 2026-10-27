use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{PaletteConfig, PaletteShowProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `palette show` can apply: the `.vxlconfig` layers'
/// `palette.show.profiles`, the user's first and the working directory's last.
pub fn load_palette_show_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<PaletteShowProfile>> {
    load_profile_set(
        dependencies,
        "palette",
        BTreeMap::new(),
        |config: PaletteConfig| config.show.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::load_palette_show_profile_set};

    #[test]
    fn the_layers_load_in_application_order_with_no_built_ins() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "palette": { "show": { "profiles": {
                        "pbr": { "properties": [{ "property": "baseColor" }] },
                    } } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "palette": { "show": { "profiles": {
                        "pbr": { "properties": [{ "property": "metallic" }, { "property": "roughness" }] },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_palette_show_profile_set(&cascade).unwrap();

        assert_eq!(profiles.get("the test", "pbr").unwrap().properties.len(), 2);

        let groups: Vec<_> = profiles
            .by_origin()
            .into_iter()
            .map(|(origin, profiles)| {
                let names: Vec<_> = profiles.into_iter().map(|(name, _)| name).collect();

                format!("{origin}: {}", names.join(" "))
            })
            .collect();
        assert_eq!(groups, ["/repo/.vxlconfig: pbr"]);
    }

    #[test]
    fn a_broken_selector_errors_naming_the_file() {
        let cascade = Cascade::new(
            true,
            &[(
                "/repo/.vxlconfig",
                r#"{ "palette": { "show": { "profiles": {
                    "pbr": { "properties": [{ "property": "baseColor", "reading": "rainbow" }] },
                } } } }"#,
            )],
        );

        let error = load_palette_show_profile_set(&cascade)
            .unwrap_err()
            .to_string();
        assert!(error.contains("`.vxlconfig`"), "{error}");
    }
}
