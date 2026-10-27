use crate::{DecodeSdfjJson, Error, Result};
use sdfj::SdfjFile;

/// Decodes `.sdfj` JSON bytes into an [`SdfjFile`] through `dependencies`.
pub fn from_sdfj_file_bytes<D: DecodeSdfjJson>(dependencies: &D, bytes: &[u8]) -> Result<SdfjFile> {
    dependencies.decode_sdfj_json(bytes).map_err(Error::Json)
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        DependenciesImpl, Error, from_sdfj_file_bytes, from_sdfj_or_sdfjz_file_bytes,
        from_sdfjz_file_bytes, to_sdfj_file_bytes, to_sdfj_pretty_file_bytes, to_sdfjz_file_bytes,
        unwrap_sdfjz,
    };
    use flate2::Crc;
    use sdfj::{
        SDFJ_VERSION, SdfjFile, SdfjMap, SdfjMapEntry, SdfjMaterial, SdfjNames, SdfjNode,
        SdfjObject, SdfjPropertyValue, SdfjShape3d, SdfjStep, SdfjTaggedValue, SdfjValue,
    };

    /// One sphere under one material, as one part.
    fn document() -> SdfjFile {
        SdfjFile {
            version: SDFJ_VERSION,
            shapes3d: vec![SdfjShape3d::Sphere {
                center: [0.0, 0.5, 0.0],
                radius: 0.25,
            }],
            shapes2d: Vec::new(),
            fonts: Vec::new(),
            materials: vec![SdfjMaterial::Material {
                properties: SdfjMap::new(vec![SdfjMapEntry {
                    key: "baseColor".to_owned(),
                    value: SdfjPropertyValue::Text("#C8B8A0".to_owned()),
                }]),
            }],
            shades: Vec::new(),
            patterns: Vec::new(),
            steps: vec![SdfjStep::Add {
                name: "ball".to_owned(),
                shape: 0,
                material: Some(0),
                pattern: None,
            }],
            objects: vec![SdfjObject {
                name: "ball".to_owned(),
                steps: vec![0],
            }],
            nodes: vec![SdfjNode {
                name: "ball".to_owned(),
                pivot: None,
                offset: None,
                rotation: Vec::new(),
                joint: false,
                child_objects: vec![0],
                child_nodes: Vec::new(),
            }],
            root_nodes: vec![0],
            names: SdfjNames::default(),
        }
    }

    #[test]
    fn compact_and_pretty_round_trip() {
        let file = document();

        for bytes in [
            to_sdfj_file_bytes(&DependenciesImpl, &file).unwrap(),
            to_sdfj_pretty_file_bytes(&DependenciesImpl, &file).unwrap(),
        ] {
            assert_eq!(bytes.last(), Some(&b'\n'));

            assert_eq!(
                from_sdfj_file_bytes(&DependenciesImpl, &bytes).unwrap(),
                file
            );
        }
    }

    #[test]
    fn sdfjz_stores_the_compact_document_as_main_sdfj() {
        let file = document();
        let member = to_sdfj_file_bytes(&DependenciesImpl, &file).unwrap();
        let zip = to_sdfjz_file_bytes(&DependenciesImpl, &file).unwrap();

        // The archive's CRC matches flate2's.
        let mut crc = Crc::new();
        crc.update(&member);
        assert_eq!(zip[14..18], crc.sum().to_le_bytes());
        assert_eq!(&zip[30..39], b"main.sdfj");

        assert_eq!(unwrap_sdfjz(&DependenciesImpl, &zip).unwrap(), member);
    }

    #[test]
    fn sdfjz_round_trips_and_detection_dispatches() {
        let file = document();
        let zip = to_sdfjz_file_bytes(&DependenciesImpl, &file).unwrap();
        assert_eq!(
            from_sdfjz_file_bytes(&DependenciesImpl, &zip).unwrap(),
            file
        );

        // Detection: leading PK -> sdfjz, leading { -> sdfj.
        assert_eq!(
            from_sdfj_or_sdfjz_file_bytes(&DependenciesImpl, &zip).unwrap(),
            file
        );
        let json = to_sdfj_file_bytes(&DependenciesImpl, &file).unwrap();
        assert_eq!(
            from_sdfj_or_sdfjz_file_bytes(&DependenciesImpl, &json).unwrap(),
            file
        );
    }

    #[test]
    fn a_malformed_sdfjz_archive_is_invalid() {
        // A truncated local header, then a whole header over a broken stream.
        let mut broken_stream = b"PK\x03\x04".to_vec();
        broken_stream.resize(30, 0);
        broken_stream.extend_from_slice(&[0xFF, 0xFF, 0xFF]);

        for bytes in [&b"PK\x03\x04short"[..], &broken_stream] {
            assert!(matches!(
                from_sdfjz_file_bytes(&DependenciesImpl, bytes),
                Err(Error::Invalid(_))
            ));
        }
    }

    // These three values mis-parse by one ULP without serde_json's
    // float_roundtrip feature, so this round trip proves the manifest carries
    // it.
    #[test]
    fn seventeen_digit_floats_save_and_load_byte_identical() {
        let mut file = document();

        file.shapes3d[0] = SdfjShape3d::Sphere {
            center: [
                0.0009105809506465125,
                0.21586050011389926,
                0.9734452903984125,
            ],
            radius: 0.25,
        };

        let bytes = to_sdfj_file_bytes(&DependenciesImpl, &file).unwrap();

        let reloaded = from_sdfj_file_bytes(&DependenciesImpl, &bytes).unwrap();

        assert_eq!(
            to_sdfj_file_bytes(&DependenciesImpl, &reloaded).unwrap(),
            bytes
        );
    }

    #[test]
    fn a_document_without_a_json_form_fails_to_write() {
        let mut nan_shape = document();
        nan_shape.shapes3d[0] = SdfjShape3d::Sphere {
            center: [f64::NAN, 0.5, 0.0],
            radius: 0.25,
        };

        let mut nan_json_value = document();
        nan_json_value.materials = vec![SdfjMaterial::Material {
            properties: SdfjMap::new(vec![SdfjMapEntry {
                key: "custom".to_owned(),
                value: SdfjPropertyValue::Tagged(SdfjTaggedValue::Json(SdfjValue::Number(
                    f64::NAN,
                ))),
            }]),
        }];

        let entry = SdfjMapEntry {
            key: "baseColor".to_owned(),
            value: SdfjPropertyValue::Text("#C8B8A0".to_owned()),
        };
        let mut repeated_property_key = document();
        repeated_property_key.materials = vec![SdfjMaterial::Material {
            properties: SdfjMap::new(vec![entry.clone(), entry]),
        }];

        for file in [nan_shape, nan_json_value, repeated_property_key] {
            assert!(matches!(
                to_sdfj_file_bytes(&DependenciesImpl, &file),
                Err(Error::Json(_))
            ));
            assert!(matches!(
                to_sdfj_pretty_file_bytes(&DependenciesImpl, &file),
                Err(Error::Json(_))
            ));
        }
    }

    #[test]
    fn undecodable_json_reports_the_parse_failure() {
        let error = from_sdfj_file_bytes(&DependenciesImpl, b"not a document").unwrap_err();

        assert!(matches!(error, Error::Json(_)));

        assert!(!error.to_string().is_empty());
    }
}
