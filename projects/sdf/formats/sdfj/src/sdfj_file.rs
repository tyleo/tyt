use crate::{
    SdfjFont, SdfjMaterial, SdfjNames, SdfjNode, SdfjObject, SdfjPattern, SdfjShades, SdfjShape2d,
    SdfjShape3d, SdfjStep,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The root of an SDF Json document. Each table holds one kind of value a
/// model can share, and entries reference each other by index. A write leaves
/// each empty table out, and a read takes a missing one as empty.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjFile {
    /// The format version. A reader takes only
    /// [`SDFJ_VERSION`](crate::SDFJ_VERSION).
    pub version: u32,

    /// The 3D shapes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub shapes3d: Vec<SdfjShape3d>,

    /// The 2D shapes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub shapes2d: Vec<SdfjShape2d>,

    /// The fonts.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub fonts: Vec<SdfjFont>,

    /// The materials.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub materials: Vec<SdfjMaterial>,

    /// The `shades` calls.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub shades: Vec<SdfjShades>,

    /// The patterns.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub patterns: Vec<SdfjPattern>,

    /// The steps.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub steps: Vec<SdfjStep>,

    /// The parts' objects.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub objects: Vec<SdfjObject>,

    /// The parts' nodes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub nodes: Vec<SdfjNode>,

    /// Indices into [`nodes`](SdfjFile::nodes): the nodes at the top of the
    /// hierarchy.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub root_nodes: Vec<usize>,

    /// The names a model reads the entries by when the document serves as a
    /// library.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjNames::is_empty")
    )]
    pub names: SdfjNames,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{
        SdfjFile, SdfjIntValue, SdfjMapEntry, SdfjMaterial, SdfjPropertyValue, SdfjStep,
        SdfjTaggedValue, SdfjValue,
    };
    use serde_json::{Value, json};

    /// The legs and seat of a chair.
    fn chair() -> Value {
        json!({
            "version": 1,
            "shapes3d": [
                {
                    "kind": "lathe",
                    "points": [[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]],
                },
                { "kind": "translate", "shape": 0, "offset": [0.2, 0, 0.2] },
                { "kind": "mirror", "shape": 1, "axes": "xz" },
                { "kind": "box", "min": [-0.25, 0.425, -0.25], "max": [0.25, 0.475, 0.25] },
            ],
            "shapes2d": [],
            "materials": [
                { "kind": "material", "properties": { "baseColor": "#8A5A2B", "roughness": 0.8 } },
                { "kind": "shade", "shades": 0, "index": 0 },
                { "kind": "shade", "shades": 0, "index": 1 },
                { "kind": "shade", "shades": 0, "index": 2 },
            ],
            "shades": [{ "base": 0, "count": 3 }],
            "patterns": [
                { "kind": "grain", "materials": [1, 2, 3], "axis": "y", "period": 0.05, "seed": 1 },
            ],
            "steps": [
                { "kind": "add", "name": "legs", "shape": 2, "pattern": 0 },
                { "kind": "add", "name": "seat", "shape": 3, "material": 0 },
            ],
            "objects": [{ "name": "chair", "steps": [0, 1] }],
            "nodes": [{ "name": "chair", "pivot": [0, 0, 0], "childObjects": [0], "childNodes": [] }],
            "rootNodes": [0],
            "names": {
                "shapes3d": { "leg": 0 },
                "shapes2d": {},
                "materials": { "walnut": 0, "wood": 0 },
                "patterns": {},
                "steps": { "legs": 0 },
                "parts": { "chair": 0 },
            },
        })
    }

    /// The chair with `edit` applied to its JSON.
    fn chair_with(edit: impl FnOnce(&mut Value)) -> Value {
        let mut chair = chair();

        edit(&mut chair);

        chair
    }

    /// The first material's properties after reading `properties`.
    fn properties(properties: Value) -> Vec<SdfjPropertyValue> {
        let chair = chair_with(|chair| chair["materials"][0]["properties"] = properties);

        let file: SdfjFile = serde_json::from_value(chair).unwrap();

        let SdfjMaterial::Material { properties } = file.materials[0].clone() else {
            panic!("the first material holds properties");
        };

        properties
            .into_entries()
            .into_iter()
            .map(|entry| entry.value)
            .collect()
    }

    #[test]
    fn a_document_round_trips() {
        let file: SdfjFile = serde_json::from_value(chair()).unwrap();

        let text = serde_json::to_string(&file).unwrap();

        assert_eq!(serde_json::from_str::<SdfjFile>(&text).unwrap(), file);

        assert_eq!(
            file.steps[0],
            SdfjStep::Add {
                name: "legs".to_owned(),
                shape: 2,
                material: None,
                pattern: Some(0),
            }
        );

        assert_eq!(
            file.names.materials.entries(),
            [
                SdfjMapEntry {
                    key: "walnut".to_owned(),
                    value: 0,
                },
                SdfjMapEntry {
                    key: "wood".to_owned(),
                    value: 0,
                },
            ]
        );
    }

    #[test]
    fn a_key_left_out_of_an_entry_stays_out_on_write() {
        let file: SdfjFile = serde_json::from_value(chair()).unwrap();

        let value = serde_json::to_value(&file).unwrap();

        assert_eq!(value["steps"][0].get("material"), None);

        assert_eq!(value["shades"][0].get("spread"), None);

        assert_eq!(value["nodes"][0].get("offset"), None);
    }

    #[test]
    fn unknown_and_missing_keys_fail_to_read() {
        let documents = [
            chair_with(|chair| chair["extra"] = json!([])),
            chair_with(|chair| {
                chair.as_object_mut().unwrap().remove("version");
            }),
            chair_with(|chair| chair["shapes3d"][3]["center"] = json!([0, 0, 0])),
            chair_with(|chair| chair["nodes"][0]["extra"] = json!(1)),
            chair_with(|chair| chair["shades"][0]["extra"] = json!(1)),
            chair_with(|chair| chair["names"]["nodes"] = json!({})),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn a_missing_table_or_names_map_reads_as_empty() {
        let file: SdfjFile = serde_json::from_value(json!({ "version": 1 })).unwrap();

        assert!(file.shapes3d.is_empty());
        assert!(file.root_nodes.is_empty());
        assert!(file.names.is_empty());

        let chair = chair_with(|chair| {
            chair["names"].as_object_mut().unwrap().remove("parts");
        });

        let file: SdfjFile = serde_json::from_value(chair).unwrap();

        assert!(file.names.parts.is_empty());
    }

    #[test]
    fn a_write_leaves_each_empty_table_and_names_map_out() {
        let file: SdfjFile = serde_json::from_value(chair()).unwrap();

        let value = serde_json::to_value(&file).unwrap();

        assert_eq!(value.get("shapes2d"), None);

        assert_eq!(value["names"].get("patterns"), None);

        let empty: SdfjFile = serde_json::from_value(json!({ "version": 1 })).unwrap();

        assert_eq!(
            serde_json::to_value(&empty).unwrap(),
            json!({ "version": 1 })
        );
    }

    #[test]
    fn an_unknown_kind_fails_to_read() {
        let chair = chair_with(|chair| chair["shapes3d"][3]["kind"] = json!("cube"));

        assert!(serde_json::from_value::<SdfjFile>(chair).is_err());
    }

    #[test]
    fn an_explicit_null_fails_to_read() {
        let documents = [
            chair_with(|chair| chair["shapes3d"][3]["round"] = Value::Null),
            chair_with(|chair| chair["nodes"][0]["pivot"] = Value::Null),
            chair_with(|chair| chair["steps"][0]["material"] = Value::Null),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn an_index_reads_only_as_a_whole_number() {
        let documents = [
            chair_with(|chair| chair["steps"][0]["shape"] = json!(2.0)),
            chair_with(|chair| chair["rootNodes"] = json!([-1])),
            chair_with(|chair| chair["names"]["steps"]["legs"] = json!("0")),
        ];

        for document in documents {
            assert!(serde_json::from_value::<SdfjFile>(document).is_err());
        }
    }

    #[test]
    fn each_property_form_reads_as_its_value() {
        let values = properties(json!({
            "bool": true,
            "int": { "kind": "int", "value": 3 },
            "ints": { "kind": "int", "value": [1, 2] },
            "json": { "kind": "json", "value": { "k": null } },
            "number": 0.5,
            "numbers": [0.5, 1],
            "text": "#FFFFFF",
        }));

        assert_eq!(
            values,
            [
                SdfjPropertyValue::Bool(true),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::Number(3.0))),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Int(SdfjIntValue::NumberArray(vec![
                    1.0, 2.0
                ]))),
                SdfjPropertyValue::Tagged(SdfjTaggedValue::Json(
                    serde_json::from_value::<SdfjValue>(json!({ "k": null })).unwrap()
                )),
                SdfjPropertyValue::Number(0.5),
                SdfjPropertyValue::NumberArray(vec![0.5, 1.0]),
                SdfjPropertyValue::Text("#FFFFFF".to_owned()),
            ]
        );
    }

    #[test]
    fn a_malformed_tagged_value_fails_to_read() {
        let documents = [
            json!({ "p": { "kind": "float", "value": 3 } }),
            json!({ "p": { "kind": "int", "value": "3" } }),
            json!({ "p": { "kind": "int", "value": 3, "extra": 1 } }),
            json!({ "p": null }),
        ];

        for document in documents {
            let chair = chair_with(|chair| chair["materials"][0]["properties"] = document);

            assert!(serde_json::from_value::<SdfjFile>(chair).is_err());
        }
    }
}
