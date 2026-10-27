use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{SdfDocConfig, SdfDocVoxelizeProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `sdf-doc voxelize` can apply from the `.vxlconfig` layers'
/// `sdfDoc.voxelize.profiles`. The user's layer loads first and the working
/// directory's last.
pub fn load_sdf_doc_voxelize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<SdfDocVoxelizeProfile>> {
    load_profile_set(
        dependencies,
        "sdfDoc",
        BTreeMap::new(),
        |config: SdfDocConfig| config.voxelize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::load_sdf_doc_voxelize_profile_set};

    #[test]
    fn the_layers_load_from_the_sdf_doc_voxelize_section() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "sdfDoc": { "voxelize": { "profiles": {
                        "props": { "voxelSize": 0.05 },
                    } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "sdfDoc": { "voxelize": { "profiles": {
                        "props": { "voxelSize": 0.025, "report": true },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_sdf_doc_voxelize_profile_set(&cascade).unwrap();
        let props = profiles.get("the test", "props").unwrap();
        assert_eq!(props.voxel_size.unwrap().0, 0.025);
        assert_eq!(props.report, Some(true));
    }
}
