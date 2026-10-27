use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{PaletteConfig, PaletteEditProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `palette edit` can apply: the `.vxlconfig` layers'
/// `palette.edit.profiles`, the user's first and the working directory's last.
pub fn load_palette_edit_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<PaletteEditProfile>> {
    load_profile_set(
        dependencies,
        "palette",
        BTreeMap::new(),
        |config: PaletteConfig| config.edit.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::load_palette_edit_profile_set};

    #[test]
    fn the_layers_load_from_the_palette_edit_section() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "palette": { "edit": { "profiles": {
                        "tags": { "values": ["rust = tag == \"rust\""] },
                    } } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "palette": { "edit": { "profiles": {
                        "tags": { "values": ["rust = false", "chrome = false"] },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_palette_edit_profile_set(&cascade).unwrap();

        assert_eq!(profiles.get("the test", "tags").unwrap().values.len(), 2);
    }
}
