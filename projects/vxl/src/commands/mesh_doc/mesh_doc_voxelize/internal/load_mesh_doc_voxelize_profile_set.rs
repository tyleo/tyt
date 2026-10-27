use crate::{
    ProfileSet, ResolvePrefsPaths, Result,
    commands::{MeshDocConfig, MeshDocVoxelizeProfile},
    load_profile_set,
};
use std::collections::BTreeMap;
use ty_preferences::Dependencies as PreferencesDependencies;

/// The profiles `mesh-doc voxelize` can apply: the `.vxlconfig` layers'
/// `meshDoc.voxelize.profiles`, the user's first and the working directory's
/// last.
pub fn load_mesh_doc_voxelize_profile_set(
    dependencies: &(impl PreferencesDependencies + ResolvePrefsPaths),
) -> Result<ProfileSet<MeshDocVoxelizeProfile>> {
    load_profile_set(
        dependencies,
        "meshDoc",
        BTreeMap::new(),
        |config: MeshDocConfig| config.voxelize.profiles,
    )
}

#[cfg(test)]
mod tests {
    use crate::{Cascade, commands::load_mesh_doc_voxelize_profile_set};
    use voxsmith::utilities::GridResolution;

    #[test]
    fn the_layers_load_from_the_mesh_doc_voxelize_section() {
        let cascade = Cascade::new(
            true,
            &[
                (
                    "/home/.vxlconfig",
                    r#"{ "meshDoc": { "voxelize": { "profiles": {
                        "fine": { "voxelSize": 0.1 },
                    } } } }"#,
                ),
                (
                    "/repo/sub/.vxlconfig",
                    r#"{ "meshDoc": { "voxelize": { "profiles": {
                        "fine": { "voxelSize": 0.05 },
                    } } } }"#,
                ),
            ],
        );

        let profiles = load_mesh_doc_voxelize_profile_set(&cascade).unwrap();
        let fine = profiles.get("the test", "fine").unwrap();
        assert_eq!(
            fine.grid_resolution().unwrap(),
            Some(GridResolution::VoxelSize(0.05))
        );
    }
}
