use crate::{
    NamedCliValue, PositiveF64, Profile, ProfileDescription, Result,
    commands::{ResolutionEntry, profile_grid_resolution},
};
use serde::Deserialize;
use voxsmith::utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame};

/// An `sdf-doc voxelize` profile, whose elements each mirror a flag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct SdfDocVoxelizeProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--resolution`.
    pub(crate) resolution: Option<ResolutionEntry>,

    /// Mirrors `--voxel-size`.
    pub(crate) voxel_size: Option<PositiveF64>,

    /// Mirrors `--frame`.
    pub(crate) frame: Option<NamedCliValue<VoxelFrame>>,

    /// Mirrors `--fill-mode`.
    pub(crate) fill_mode: Option<NamedCliValue<FillMode>>,

    /// Mirrors `--flatten`.
    pub(crate) flatten: Option<NamedCliValue<FlattenMode>>,

    /// Mirrors `--report`.
    pub(crate) report: Option<bool>,
}

impl Profile for SdfDocVoxelizeProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

impl SdfDocVoxelizeProfile {
    /// The grid resolution `resolution` or `voxelSize` sets. Errors when the
    /// profile sets both.
    pub(crate) fn grid_resolution(&self) -> Result<Option<GridResolution>> {
        profile_grid_resolution(self.resolution, self.voxel_size)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NamedCliValue, commands::SdfDocVoxelizeProfile};
    use voxsmith::utilities::{
        FillMode, FlattenMode, GridResolution, ResolutionReference, VoxelFrame,
    };

    /// The profile `json` defines.
    fn profile(json: &str) -> SdfDocVoxelizeProfile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn every_key_reads_into_its_element() {
        let profile = profile(
            r#"{
                "description": "Icons",
                "resolution": { "reference": "longest-world", "count": 16 },
                "frame": "world",
                "fillMode": "surface",
                "flatten": "objects",
                "report": false
            }"#,
        );

        assert_eq!(profile.description.as_ref().unwrap().as_str(), "Icons");
        assert_eq!(
            profile.grid_resolution().unwrap(),
            Some(GridResolution::ReferenceCount {
                reference: ResolutionReference::LongestWorld,
                count: 16,
            })
        );
        assert_eq!(profile.frame, Some(NamedCliValue(VoxelFrame::World)));
        assert_eq!(profile.fill_mode, Some(NamedCliValue(FillMode::Surface)));
        assert_eq!(profile.flatten, Some(NamedCliValue(FlattenMode::Objects)));
        assert_eq!(profile.report, Some(false));
    }

    #[test]
    fn a_resolution_beside_a_voxel_size_errors() {
        let both = profile(
            r#"{ "resolution": { "reference": "world-x", "count": 8 }, "voxelSize": 0.5 }"#,
        );
        assert!(both.grid_resolution().is_err());
    }

    #[test]
    fn a_bad_key_or_value_errors() {
        for json in [
            r#"{ "voxelSize": 0 }"#,
            r#"{ "frame": "both" }"#,
            r#"{ "flatten": true }"#,
            r#"{ "scale": "keep" }"#,
            r#"{ "output": "chair.voxj" }"#,
        ] {
            assert!(
                serde_json::from_str::<SdfDocVoxelizeProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
