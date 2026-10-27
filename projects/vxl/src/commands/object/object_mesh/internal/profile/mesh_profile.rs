use crate::{
    NamedCliValue, Profile, ProfileDescription,
    commands::{
        BoundNames, ComputeIndexEntry, ExtraEntry, FileEntries, MaterialEntry, PrimitiveEntry,
        TextureShapeEntry,
    },
};
use serde::Deserialize;
use std::collections::BTreeMap;
use voxsmith::operations::object::Method;

/// A profile, each element mirroring an `object mesh` flag. An unknown key
/// errors at load.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct MeshProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--values-from` per entry. Writers never travel.
    pub(crate) values_from: Vec<String>,

    /// Mirrors `--compute-index`, each domain key holding its bound names.
    pub(crate) compute_index: ComputeIndexEntry,

    /// Mirrors `--compute-occlusion` per bound name.
    pub(crate) compute_occlusion: BoundNames,

    /// Mirrors `--compute-voxel-position` per bound name.
    pub(crate) compute_voxel_position: BoundNames,

    /// Mirrors `--value` per entry.
    pub(crate) values: Vec<String>,

    /// Mirrors `--scene-scale`.
    pub(crate) scene_scale: Option<f64>,

    /// Mirrors `--method`.
    pub(crate) method: Option<NamedCliValue<Method>>,

    /// Mirrors `--texture-shape`.
    pub(crate) texture_shape: Option<TextureShapeEntry>,

    /// The files written beside the mesh.
    pub(crate) files: FileEntries,

    /// Mirrors `--material-count` by length. Absent, count 0.
    pub(crate) materials: Vec<MaterialEntry>,

    /// Mirrors `--primitive` per entry. Absent, the implicit primitive.
    pub(crate) primitives: Vec<PrimitiveEntry>,

    /// Mirrors the `--write-mesh-extra-*` flags, an entry per name.
    pub(crate) mesh_extras: BTreeMap<String, ExtraEntry>,
}

impl Profile for MeshProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        NamedCliValue,
        commands::{ExtraEntry, MeshProfile, SlotEntry, TextureShapeEntry, ValueEntry},
    };
    use voxsmith::operations::object::{Method, TextureShape, Transfer};

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: MeshProfile = serde_json::from_str(
            r#"{
                "valuesFrom": ["defaults"],
                "computeIndex": { "swatch": "swatchIndex", "face": ["a", "b"] },
                "computeOcclusion": "ao",
                "computeVoxelPosition": ["at"],
                "values": ["albedo = baseColor"],
                "sceneScale": 0.1,
                "method": "culled",
                "textureShape": 32,
                "files": {
                    "png": { "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" } },
                    "json": { "{file-stem}.json": { "ior": { "transfer": "linear", "value": "ior" } } }
                },
                "materials": [
                    {
                        "name": "body",
                        "uvs": ["swatch"],
                        "slots": {
                            "baseColorTexture": { "kind": "value", "value": "albedo" },
                            "occlusionTexture": { "kind": "file", "file": "{file-stem}-orm.png" }
                        },
                        "extras": { "heat": { "kind": "image-file", "file": "{file-stem}-orm.png" } }
                    }
                ],
                "primitives": [
                    {
                        "name": "body",
                        "select": "solid",
                        "material": 0,
                        "normal": false,
                        "uvs": ["face"],
                        "builtins": { "COLOR_0": "albedo" },
                        "customs": { "_HEAT": { "transfer": "linear", "value": "heat" } }
                    }
                ],
                "meshExtras": { "accent": { "kind": "json-value", "transfer": "srgb", "value": "accent" } }
            }"#,
        )
        .unwrap();

        assert_eq!(profile.values_from, ["defaults"]);
        assert_eq!(
            profile.compute_index.bindings().collect::<Vec<_>>().len(),
            3
        );
        assert_eq!(profile.compute_occlusion.0, ["ao"]);
        assert_eq!(profile.compute_voxel_position.0, ["at"]);
        assert_eq!(profile.scene_scale, Some(0.1));
        assert_eq!(profile.method, Some(NamedCliValue(Method::Culled)));
        assert_eq!(
            profile.texture_shape,
            Some(TextureShapeEntry(TextureShape::Exact(32)))
        );
        assert_eq!(
            profile.files.png["{file-stem}-orm.png"],
            ValueEntry {
                transfer: NamedCliValue(Transfer::Linear),
                value: "orm".to_owned(),
            }
        );
        assert_eq!(profile.files.json["{file-stem}.json"]["ior"].value, "ior");

        let material = &profile.materials[0];
        assert_eq!(material.name.as_deref(), Some("body"));
        assert_eq!(material.uvs.as_deref(), Some(&["swatch".to_owned()][..]));
        assert_eq!(
            material.slots["occlusionTexture"],
            SlotEntry::File {
                file: "{file-stem}-orm.png".to_owned()
            }
        );
        assert!(matches!(
            material.extras["heat"],
            ExtraEntry::ImageFile { .. }
        ));

        let primitive = &profile.primitives[0];
        assert_eq!(primitive.material, Some(0));
        assert_eq!(primitive.normal, Some(false));
        assert_eq!(primitive.builtins["COLOR_0"], "albedo");
        assert_eq!(primitive.customs["_HEAT"].value, "heat");

        assert!(matches!(
            profile.mesh_extras["accent"],
            ExtraEntry::JsonValue { .. }
        ));
    }

    #[test]
    fn the_empty_profile_takes_every_default() {
        assert_eq!(
            serde_json::from_str::<MeshProfile>("{}").unwrap(),
            MeshProfile::default()
        );
    }

    #[test]
    fn an_unknown_key_errors_at_every_depth() {
        assert!(serde_json::from_str::<MeshProfile>(r#"{ "value": [] }"#).is_err());
        assert!(
            serde_json::from_str::<MeshProfile>(r#"{ "computeIndex": { "edge": "e" } }"#).is_err()
        );
        assert!(serde_json::from_str::<MeshProfile>(r#"{ "files": { "gif": {} } }"#).is_err());
        assert!(serde_json::from_str::<MeshProfile>(r#"{ "materials": [{ "uv": [] }] }"#).is_err());
        assert!(
            serde_json::from_str::<MeshProfile>(
                r#"{ "materials": [{ "slots": { "a": { "kind": "value", "value": "v", "file": "f" } } }] }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<MeshProfile>(
                r#"{ "meshExtras": { "a": { "kind": "json-file", "file": "f", "value": "v" } } }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<MeshProfile>(r#"{ "primitives": [{ "builtin": {} }] }"#)
                .is_err()
        );
    }

    #[test]
    fn a_kind_outside_the_vocabulary_errors() {
        assert!(
            serde_json::from_str::<MeshProfile>(
                r#"{ "materials": [{ "slots": { "a": { "kind": "image", "value": "v" } } }] }"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<MeshProfile>(
                r#"{ "meshExtras": { "a": { "kind": "image", "value": "v", "transfer": "srgb" } } }"#
            )
            .is_err()
        );
    }
}
