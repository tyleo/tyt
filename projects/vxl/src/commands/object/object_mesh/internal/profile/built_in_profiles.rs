use crate::commands::MeshProfile;
use jsonc_parser::{ParseOptions, parse_to_serde_value};
use std::collections::BTreeMap;

/// The built-in profiles, parsed from the jsonc map the binary embeds.
pub fn built_in_profiles() -> BTreeMap<String, MeshProfile> {
    let options = ParseOptions {
        allow_comments: true,
        allow_loose_object_property_names: false,
        allow_trailing_commas: true,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
    };

    parse_to_serde_value(include_str!("built_in_profiles.jsonc"), &options)
        .expect("the embedded profiles are well-formed jsonc following the schema")
}

#[cfg(test)]
mod tests {
    use crate::{Profile, commands::built_in_profiles};
    use vox_value_language::parse;

    #[test]
    fn the_six_built_ins_load_and_their_values_parse() {
        let profiles = built_in_profiles();

        assert_eq!(
            profiles.keys().collect::<Vec<_>>(),
            ["albedo", "defaults", "emissive", "glass", "orm", "pbr"]
        );

        for (name, profile) in &profiles {
            for fragment in &profile.values {
                parse(&format!("{fragment};")).unwrap_or_else(|error| {
                    panic!("the profile `{name}` holds `{fragment}`: {error}")
                });
            }
        }
    }

    #[test]
    fn pbr_imports_the_three_maps_and_writes_the_whole_material() {
        let profiles = built_in_profiles();
        let pbr = &profiles["pbr"];

        assert_eq!(pbr.values_from, ["albedo", "orm", "emissive"]);
        assert!(pbr.values.is_empty());
        assert_eq!(pbr.materials[0].slots.len(), 6);
    }

    #[test]
    fn every_built_in_carries_a_description() {
        for (name, profile) in built_in_profiles() {
            assert!(profile.description().is_some(), "{name}");
        }
    }
}
