use crate::{NamedCliValue, Profile, ProfileDescription};
use serde::Deserialize;
use std::num::NonZeroUsize;
use voxsmith::utilities::{AlphaMode, ColorSpace, Dither, PropertyInterpretation, ReductionMethod};

/// A quantize profile, each element mirroring a flag `palette quantize` and
/// `object voxels quantize` share.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct QuantizeProfile {
    /// Printed beside the profile name in the profile listings.
    pub(crate) description: Option<ProfileDescription>,

    /// Mirrors `--max-materials`.
    pub(crate) max_materials: Option<NonZeroUsize>,

    /// Mirrors `--property`.
    pub(crate) property: Option<String>,

    /// Mirrors `--interpret-property`.
    pub(crate) interpret_property: Option<NamedCliValue<PropertyInterpretation>>,

    /// Mirrors `--alpha`.
    pub(crate) alpha: Option<NamedCliValue<AlphaMode>>,

    /// Mirrors `--partition` per entry.
    pub(crate) partition: Vec<String>,

    /// Mirrors `--method`.
    pub(crate) method: Option<NamedCliValue<ReductionMethod>>,

    /// Mirrors `--space`.
    pub(crate) space: Option<NamedCliValue<ColorSpace>>,

    /// Mirrors `--dither`.
    pub(crate) dither: Option<NamedCliValue<Dither>>,
}

impl Profile for QuantizeProfile {
    fn description(&self) -> Option<&ProfileDescription> {
        self.description.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{NamedCliValue, QuantizeProfile};
    use voxsmith::utilities::{AlphaMode, Dither, PropertyInterpretation};

    #[test]
    fn every_key_reads_into_its_element() {
        let profile: QuantizeProfile = serde_json::from_str(
            r#"{
                "maxMaterials": 16,
                "property": "roughness",
                "interpretProperty": "numeric",
                "alpha": "ignore",
                "partition": ["metallic"],
                "method": "kmeans",
                "space": "lab",
                "dither": "ordered"
            }"#,
        )
        .unwrap();
        assert_eq!(profile.max_materials.unwrap().get(), 16);
        assert_eq!(profile.property.as_deref(), Some("roughness"));
        assert_eq!(
            profile.interpret_property,
            Some(NamedCliValue(PropertyInterpretation::Numeric))
        );
        assert_eq!(profile.alpha, Some(NamedCliValue(AlphaMode::Ignore)));
        assert_eq!(profile.partition, ["metallic"]);
        assert_eq!(profile.dither, Some(NamedCliValue(Dither::Ordered)));
    }

    #[test]
    fn a_bad_key_or_value_errors() {
        for json in [
            r#"{ "maxMaterials": 0 }"#,
            r#"{ "method": "voronoi" }"#,
            r#"{ "index": 1 }"#,
            r#"{ "keepUnusedValues": true }"#,
        ] {
            assert!(
                serde_json::from_str::<QuantizeProfile>(json).is_err(),
                "{json}"
            );
        }
    }
}
