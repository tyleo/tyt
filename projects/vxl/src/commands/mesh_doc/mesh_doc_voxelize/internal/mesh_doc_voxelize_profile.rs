use crate::{
    NamedCliValue, NoneOr, PositiveF64, Profile, ProfileDescription, Result, Rgba,
    commands::{ResolutionEntry, profile_grid_resolution},
};
use serde::Deserialize;
use voxsmith::{
    operations::mesh_doc::{MaterialMode, OutOfRangeProperty, SurfaceMode, VoxelScale},
    utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame},
};

/// A `mesh-doc voxelize` profile, each element mirroring a flag.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MeshDocVoxelizeProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--resolution`.
    pub(crate) resolution: Option<ResolutionEntry>,

    /// Mirrors `--voxel-size`.
    pub(crate) voxel_size: Option<PositiveF64>,

    /// Mirrors `--frame`.
    pub(crate) frame: Option<NamedCliValue<VoxelFrame>>,

    /// Mirrors `--scale`.
    pub(crate) scale: Option<NamedCliValue<VoxelScale>>,

    /// Mirrors `--flatten`.
    pub(crate) flatten: Option<NamedCliValue<FlattenMode>>,

    /// Mirrors `--fill-mode`.
    pub(crate) fill_mode: Option<NamedCliValue<FillMode>>,

    /// Mirrors `--surface-mode`.
    pub(crate) surface_mode: Option<NamedCliValue<SurfaceMode>>,

    /// Mirrors `--material-mode`.
    pub(crate) material_mode: Option<NamedCliValue<MaterialMode>>,

    /// Mirrors `--fill-color`.
    pub(crate) fill_color: Option<NoneOr<Rgba>>,

    /// Mirrors `--out-of-range-property`.
    pub(crate) out_of_range_property: Option<NamedCliValue<OutOfRangeProperty>>,
}

impl Profile for MeshDocVoxelizeProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

impl MeshDocVoxelizeProfile {
    /// The grid resolution `resolution` or `voxelSize` sets. Errors when the
    /// profile sets both.
    pub(crate) fn grid_resolution(&self) -> Result<Option<GridResolution>> {
        profile_grid_resolution(self.resolution, self.voxel_size)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NamedCliValue, NoneOr, Rgba, commands::MeshDocVoxelizeProfile};
    use voxsmith::{
        operations::mesh_doc::{MaterialMode, OutOfRangeProperty, SurfaceMode, VoxelScale},
        utilities::{FillMode, FlattenMode, GridResolution, ResolutionReference, VoxelFrame},
    };

    /// The profile `json` defines.
    fn profile(json: &str) -> MeshDocVoxelizeProfile {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn every_key_reads_into_its_element() {
        let profile = profile(
            r##"{
                "resolution": { "reference": "longest-object", "count": 32 },
                "frame": "local",
                "scale": "keep",
                "flatten": "nodes",
                "fillMode": "surface",
                "surfaceMode": "triangle-cover",
                "materialMode": "flat",
                "fillColor": "#ff0000",
                "outOfRangeProperty": "clamp"
            }"##,
        );

        assert_eq!(
            profile.grid_resolution().unwrap(),
            Some(GridResolution::ReferenceCount {
                reference: ResolutionReference::LongestObject,
                count: 32,
            })
        );
        assert_eq!(profile.frame, Some(NamedCliValue(VoxelFrame::Local)));
        assert_eq!(profile.scale, Some(NamedCliValue(VoxelScale::Keep)));
        assert_eq!(profile.flatten, Some(NamedCliValue(FlattenMode::Nodes)));
        assert_eq!(profile.fill_mode, Some(NamedCliValue(FillMode::Surface)));
        assert_eq!(
            profile.surface_mode,
            Some(NamedCliValue(SurfaceMode::TriangleCover))
        );
        assert_eq!(
            profile.material_mode,
            Some(NamedCliValue(MaterialMode::Flat))
        );
        assert_eq!(
            profile.fill_color,
            Some(NoneOr::Value(Rgba([255, 0, 0, 255])))
        );
        assert_eq!(
            profile.out_of_range_property,
            Some(NamedCliValue(OutOfRangeProperty::Clamp))
        );
    }

    #[test]
    fn a_voxel_size_reads_into_the_grid_resolution() {
        assert_eq!(
            profile(r#"{ "voxelSize": 0.25 }"#)
                .grid_resolution()
                .unwrap(),
            Some(GridResolution::VoxelSize(0.25))
        );
        assert_eq!(profile("{}").grid_resolution().unwrap(), None);
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
            r#"{ "resolution": { "reference": "longest", "count": 8 } }"#,
            r#"{ "resolution": { "reference": "world-x", "count": 0 } }"#,
            r#"{ "resolution": { "reference": "world-x" } }"#,
            r#"{ "voxelSize": 0 }"#,
            r#"{ "fillColor": "white" }"#,
            r#"{ "frame": "scene" }"#,
            r#"{ "from": "glb" }"#,
            r#"{ "format": "zip" }"#,
        ] {
            assert!(
                serde_json::from_str::<MeshDocVoxelizeProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
