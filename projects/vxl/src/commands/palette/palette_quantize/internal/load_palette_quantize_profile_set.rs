use crate::{
    ProfileSet, QuantizeProfile, ResolvePrefsPaths, Result, commands::PaletteConfig,
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `palette quantize` can apply: the `.vxlconfig` layers'
/// `palette.quantize.profiles`, the user's first and the working directory's
/// last.
pub fn load_palette_quantize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<QuantizeProfile>> {
    load_profile_set(
        dependencies,
        "palette",
        BTreeMap::new(),
        |config: PaletteConfig| config.quantize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::load_palette_quantize_profile_set};

    #[test]
    fn the_layers_load_from_the_palette_quantize_section() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "palette": { "quantize": { "profiles": {
                        "retro": { "maxMaterials": 16 },
                    } } } }"#,
                ),
                (
                    "/repo/.vxlconfig",
                    r#"{ "palette": { "quantize": { "profiles": {
                        "retro": { "maxMaterials": 8 },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_palette_quantize_profile_set(&cascade).unwrap();
        let retro = profiles.get("the test", "retro").unwrap();
        assert_eq!(retro.max_materials.unwrap().get(), 8);
    }
}
