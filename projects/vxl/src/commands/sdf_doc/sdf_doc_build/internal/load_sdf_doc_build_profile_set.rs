use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{SdfDocBuildProfile, SdfDocConfig},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `sdf-doc build` can apply from the `.vxlconfig` layers'
/// `sdfDoc.build.profiles`. The user's layer loads first and the working
/// directory's last.
pub fn load_sdf_doc_build_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<SdfDocBuildProfile>> {
    load_profile_set(
        dependencies,
        "sdfDoc",
        BTreeMap::new(),
        |config: SdfDocConfig| config.build.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, NamedCliValue, commands::load_sdf_doc_build_profile_set};
    use sdfj_builder::JavaScriptRuntime;

    #[test]
    fn the_layers_load_from_the_sdf_doc_build_section() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "sdfDoc": { "build": { "profiles": {
                        "fast": { "runtime": "bun" },
                    } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "sdfDoc": { "build": { "profiles": {
                        "fast": { "runtime": "deno" },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_sdf_doc_build_profile_set(&cascade).unwrap();
        let fast = profiles.get("the test", "fast").unwrap();
        assert_eq!(fast.runtime, Some(NamedCliValue(JavaScriptRuntime::Deno)));
    }
}
