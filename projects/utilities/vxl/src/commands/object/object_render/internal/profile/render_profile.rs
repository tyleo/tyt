use crate::{
    NamedCliValue, PositiveF64, Profile, ProfileDescription,
    commands::{Background, LightEntry, NonNegativeF64, ViewEntry},
};
use serde::Deserialize;
use std::{collections::BTreeMap, num::NonZeroU32};
use voxsmith::operations::object::RenderOcclusion;

/// A render profile. Each element mirrors an `object render` flag. An
/// unknown key errors at load.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct RenderProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--width`.
    pub(crate) width: Option<NonZeroU32>,

    /// Mirrors `--height`.
    pub(crate) height: Option<NonZeroU32>,

    /// Mirrors `--background`.
    pub(crate) background: Option<Background>,

    /// Mirrors `--occlusion`.
    pub(crate) occlusion: Option<NamedCliValue<RenderOcclusion>>,

    /// Mirrors `--voxel-size`.
    pub(crate) voxel_size: Option<PositiveF64>,

    /// Mirrors `--bloom-strength`.
    pub(crate) bloom_strength: Option<NonNegativeF64>,

    /// Mirrors `--bloom-radius`.
    pub(crate) bloom_radius: Option<PositiveF64>,

    /// Mirrors `--bloom-threshold`.
    pub(crate) bloom_threshold: Option<NonNegativeF64>,

    /// Mirrors `--views-from` per entry. Only the views travel.
    pub(crate) views_from: Vec<String>,

    /// Mirrors the `--view-*` flags, keyed by the name that suffixes the file.
    pub(crate) views: BTreeMap<String, ViewEntry>,

    /// Mirrors `--lights-from` per entry. Only the rig travels.
    pub(crate) lights_from: Vec<String>,

    /// Mirrors the `--light-*` flags. A light's list position gives its index.
    pub(crate) lights: Vec<LightEntry>,
}

impl Profile for RenderProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        NamedCliValue, PositiveF64, ProfileDescription,
        commands::{
            Background, DistanceEntry, LightEntry, NonNegativeF64, PoseTransformEntry,
            PositionTransformEntry, ProjectionKind, RenderProfile, RotationEntry,
            RotationTransformEntry, SpotTransformEntry, SrgbColor,
        },
    };
    use ty_math::{TyAngleUnit, TySrgbU8};
    use voxsmith::operations::object::{FitOrFixed, RenderOcclusion, RenderShadow};

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: RenderProfile = serde_json::from_str(
            r##"{
                "description": "A test rig",
                "width": 640,
                "height": 480,
                "background": "#202020",
                "occlusion": "none",
                "voxelSize": 0.1,
                "bloomStrength": 1.5,
                "bloomRadius": 0.05,
                "bloomThreshold": 0.8,
                "viewsFrom": ["front", "top"],
                "lightsFrom": ["studio"],
                "views": {
                    "hero": {
                        "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 },
                        "fov": 50,
                        "select": ["house/**"]
                    },
                    "plan": {
                        "transform": {
                            "kind": "subject",
                            "position": [0, 5, 0],
                            "rotation": { "kind": "look-at" }
                        },
                        "projection": "orthographic",
                        "scale": 12
                    },
                    "fixed": {
                        "transform": {
                            "kind": "world",
                            "position": [1, 2, 3],
                            "rotation": { "kind": "euler", "value": [0, 90, 0], "unit": "deg" }
                        }
                    }
                },
                "lights": [
                    {
                        "kind": "directional",
                        "transform": {
                            "kind": "camera",
                            "rotation": { "kind": "angles", "azimuth": -30, "elevation": 30 }
                        },
                        "shadow": "per-face",
                        "color": "#FFFFFF",
                        "strength": 3
                    },
                    {
                        "kind": "point",
                        "transform": { "kind": "orbit", "azimuth": 0, "elevation": 45, "distance": 4 },
                        "range": 10
                    },
                    { "kind": "hemisphere", "sky": "#8090A0", "ground": "#403020", "strength": 0.5 }
                ]
            }"##,
        )
        .unwrap();

        assert_eq!(
            profile.description.as_ref().map(ProfileDescription::as_str),
            Some("A test rig")
        );
        assert_eq!(profile.width.map(u32::from), Some(640));
        assert_eq!(profile.height.map(u32::from), Some(480));
        assert_eq!(
            profile.background,
            Some(Background::Color(SrgbColor(TySrgbU8::new(32, 32, 32))))
        );
        assert_eq!(
            profile.occlusion,
            Some(NamedCliValue(RenderOcclusion::None))
        );
        assert_eq!(profile.voxel_size, Some(PositiveF64(0.1)));
        assert_eq!(profile.bloom_strength, Some(NonNegativeF64(1.5)));
        assert_eq!(profile.bloom_radius, Some(PositiveF64(0.05)));
        assert_eq!(profile.bloom_threshold, Some(NonNegativeF64(0.8)));
        assert_eq!(profile.views_from, ["front", "top"]);
        assert_eq!(profile.lights_from, ["studio"]);

        let hero = &profile.views["hero"];
        assert_eq!(
            hero.transform,
            Some(PoseTransformEntry::Orbit {
                azimuth: 45.0,
                elevation: 30.0,
                distance: None,
            })
        );
        assert_eq!(hero.fov, Some(PositiveF64(50.0)));
        assert_eq!(hero.select.as_deref(), Some(&["house/**".to_owned()][..]));

        let plan = &profile.views["plan"];
        assert_eq!(
            plan.transform,
            Some(PoseTransformEntry::Subject {
                position: [0.0, 5.0, 0.0],
                rotation: RotationEntry::LookAt { target: None },
            })
        );
        assert_eq!(
            plan.projection,
            Some(NamedCliValue(ProjectionKind::Orthographic))
        );
        assert_eq!(plan.scale, Some(PositiveF64(12.0)));

        assert_eq!(
            profile.views["fixed"].transform,
            Some(PoseTransformEntry::World {
                position: [1.0, 2.0, 3.0],
                rotation: RotationEntry::Euler {
                    value: [0.0, 90.0, 0.0],
                    unit: Some(NamedCliValue(TyAngleUnit::Degrees)),
                },
            })
        );

        let [sun, lamp, sky] = profile.lights.as_slice() else {
            panic!("three lights");
        };
        assert_eq!(
            *sun,
            LightEntry::Directional {
                transform: Some(RotationTransformEntry::Camera {
                    rotation: RotationEntry::Angles {
                        azimuth: -30.0,
                        elevation: 30.0,
                    },
                }),
                shadow: Some(NamedCliValue(RenderShadow::PerFace)),
                color: Some(SrgbColor(TySrgbU8::new(255, 255, 255))),
                strength: Some(NonNegativeF64(3.0)),
            }
        );
        assert_eq!(
            *lamp,
            LightEntry::Point {
                transform: Some(PositionTransformEntry::Orbit {
                    azimuth: 0.0,
                    elevation: 45.0,
                    distance: PositiveF64(4.0),
                }),
                shadow: None,
                color: None,
                strength: None,
                range: Some(PositiveF64(10.0)),
            }
        );
        assert!(matches!(sky, LightEntry::Hemisphere { .. }));

        assert_eq!(
            serde_json::from_str::<DistanceEntry>(r#""fit""#).unwrap(),
            DistanceEntry(FitOrFixed::Fit)
        );
    }

    #[test]
    fn a_node_transform_carries_its_path() {
        let profile: RenderProfile = serde_json::from_str(
            r#"{
                "views": {
                    "ride": {
                        "transform": {
                            "kind": "node",
                            "path": "player/head",
                            "position": [0, 1, 5],
                            "rotation": { "kind": "look-at" }
                        }
                    }
                },
                "lights": [
                    {
                        "kind": "directional",
                        "transform": {
                            "kind": "node",
                            "path": "sun",
                            "rotation": { "kind": "angles", "azimuth": 0, "elevation": 90 }
                        }
                    },
                    {
                        "kind": "point",
                        "transform": { "kind": "node", "path": "lamp", "position": [0, 1, 0] }
                    }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            profile.views["ride"].transform,
            Some(PoseTransformEntry::Node {
                path: "player/head".to_owned(),
                position: [0.0, 1.0, 5.0],
                rotation: RotationEntry::LookAt { target: None },
            })
        );

        let [sun, lamp] = profile.lights.as_slice() else {
            panic!("two lights");
        };
        assert!(matches!(
            sun,
            LightEntry::Directional {
                transform: Some(RotationTransformEntry::Node { path, .. }),
                ..
            } if path == "sun"
        ));
        assert!(matches!(
            lamp,
            LightEntry::Point {
                transform: Some(PositionTransformEntry::Node {
                    path,
                    position: [0.0, 1.0, 0.0]
                }),
                ..
            } if path == "lamp"
        ));

        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "views": { "a": { "transform": { "kind": "node", "position": [0, 0, 0], "rotation": { "kind": "look-at" } } } } }"#
            )
            .is_err()
        );
    }

    #[test]
    fn a_spot_entry_carries_its_pose_cone_and_range() {
        let profile: RenderProfile = serde_json::from_str(
            r#"{
                "lights": [
                    {
                        "kind": "spot",
                        "transform": {
                            "kind": "camera",
                            "position": [0, 0, 0],
                            "rotation": { "kind": "angles", "azimuth": 0, "elevation": 0 }
                        },
                        "innerCone": 10,
                        "outerCone": 30,
                        "range": 5
                    },
                    {
                        "kind": "spot",
                        "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30, "distance": 4 }
                    }
                ]
            }"#,
        )
        .unwrap();

        let [headlight, orbiting] = profile.lights.as_slice() else {
            panic!("two lights");
        };
        assert_eq!(
            *headlight,
            LightEntry::Spot {
                transform: Some(SpotTransformEntry::Camera {
                    position: [0.0, 0.0, 0.0],
                    rotation: RotationEntry::Angles {
                        azimuth: 0.0,
                        elevation: 0.0,
                    },
                }),
                shadow: None,
                color: None,
                strength: None,
                range: Some(PositiveF64(5.0)),
                inner_cone: Some(NonNegativeF64(10.0)),
                outer_cone: Some(NonNegativeF64(30.0)),
            }
        );
        assert!(matches!(
            orbiting,
            LightEntry::Spot {
                transform: Some(SpotTransformEntry::Orbit {
                    distance: PositiveF64(4.0),
                    ..
                }),
                inner_cone: None,
                outer_cone: None,
                ..
            }
        ));

        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "lights": [{ "kind": "spot", "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } }] }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "lights": [{ "kind": "spot", "innerCone": -1 }] }"#
            )
            .is_err()
        );
    }

    #[test]
    fn the_empty_profile_takes_every_default() {
        assert_eq!(
            serde_json::from_str::<RenderProfile>("{}").unwrap(),
            RenderProfile::default()
        );
    }

    #[test]
    fn an_unknown_key_errors_at_every_depth() {
        assert!(serde_json::from_str::<RenderProfile>(r#"{ "size": 1 }"#).is_err());
        assert!(serde_json::from_str::<RenderProfile>(r#"{ "width": 0 }"#).is_err());
        assert!(serde_json::from_str::<RenderProfile>(r#"{ "bloomRadius": 0 }"#).is_err());
        assert!(serde_json::from_str::<RenderProfile>(r#"{ "bloomStrength": -1 }"#).is_err());
        assert!(
            serde_json::from_str::<RenderProfile>(r#"{ "views": { "a": { "fovy": 1 } } }"#)
                .is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "views": { "a": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0, "position": [0, 0, 0] } } } }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "views": { "a": { "transform": { "kind": "world", "position": [0, 0, 0], "rotation": { "kind": "look-at", "value": [0, 0, 0, 1] } } } } }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "lights": [{ "kind": "hemisphere", "range": 1 }] }"#
            )
            .is_err()
        );
    }

    #[test]
    fn a_kind_outside_the_vocabulary_errors() {
        assert!(
            serde_json::from_str::<RenderProfile>(r#"{ "lights": [{ "kind": "area" }] }"#).is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "views": { "a": { "transform": { "kind": "rig", "position": [0, 0, 0] } } } }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "lights": [{ "kind": "directional", "transform": { "kind": "subject", "rotation": { "kind": "look-at" } } }] }"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<RenderProfile>(r#"{ "occlusion": "traced" }"#).is_err());
        assert!(
            serde_json::from_str::<RenderProfile>(
                r#"{ "views": { "a": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0, "distance": "near" } } } }"#
            )
            .is_err()
        );
    }
}
