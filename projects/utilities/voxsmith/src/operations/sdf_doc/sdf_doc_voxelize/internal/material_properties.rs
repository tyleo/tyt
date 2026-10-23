use crate::operations::sdf_doc::{ArgumentCheck, SdfCustomValue, SdfMaterialProperties};
use meshdoc::material::{
    BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, EMISSIVE_STRENGTH_DEFAULT, IOR, IOR_DEFAULT,
    METALLIC, MaterialRange, OCCLUSION_STRENGTH, OCCLUSION_STRENGTH_DEFAULT, ROUGHNESS,
    ROUGHNESS_DEFAULT, TRANSMISSION, TRANSMISSION_DEFAULT, scalar_range,
};
use sdfcore::{SdfProperty, SdfPropertyValue, SdfValue};
use std::{collections::BTreeMap, result::Result as StdResult};
use ty_math::{TyHexColor, TyLinSrgbF64, TyLinSrgbaF64, TySrgbaU8};
use voxcore::color::lin_srgba_f64_from_srgba_u8;

/// The largest magnitude an `int` holds.
const MOST_INT: f64 = 9_007_199_254_740_991.0;

/// The properties of a `material` entry holding `properties`. Each named
/// property falls in its range, and each custom one takes a voxj kind.
pub fn material_properties(properties: &[SdfProperty]) -> StdResult<SdfMaterialProperties, String> {
    let check = ArgumentCheck::new("material");

    let mut material = SdfMaterialProperties {
        base_color: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0),
        emissive_color: TyLinSrgbF64::new(0.0, 0.0, 0.0),
        emissive_strength: EMISSIVE_STRENGTH_DEFAULT,
        ior: IOR_DEFAULT,
        metallic: 0.0,
        occlusion_strength: OCCLUSION_STRENGTH_DEFAULT,
        roughness: ROUGHNESS_DEFAULT,
        transmission: TRANSMISSION_DEFAULT,
        custom: BTreeMap::new(),
    };

    for property in properties {
        let name = property.name.as_str();
        let value = &property.value;

        match name {
            BASE_COLOR => {
                material.base_color = color(value, true)
                    .ok_or_else(|| check.fail(name, "#RRGGBB or #RRGGBBAA", shown(value)))?;
            }

            EMISSIVE_COLOR => {
                let color =
                    color(value, false).ok_or_else(|| check.fail(name, "#RRGGBB", shown(value)))?;
                material.emissive_color = TyLinSrgbF64::new(color.red, color.green, color.blue);
            }

            EMISSIVE_STRENGTH => material.emissive_strength = scalar(check, name, value)?,

            IOR => material.ior = scalar(check, name, value)?,

            METALLIC => material.metallic = scalar(check, name, value)?,

            OCCLUSION_STRENGTH => material.occlusion_strength = scalar(check, name, value)?,

            ROUGHNESS => material.roughness = scalar(check, name, value)?,

            TRANSMISSION => material.transmission = scalar(check, name, value)?,

            _ => {
                material
                    .custom
                    .insert(name.to_owned(), custom_value(check, name, value)?);
            }
        }
    }

    Ok(material)
}

/// The color `#RRGGBB` in `value` in linear light, or `#RRGGBBAA` with a
/// straight alpha when `takes_alpha`. Any other value reads `None`.
fn color(value: &SdfPropertyValue, takes_alpha: bool) -> Option<TyLinSrgbaF64> {
    let SdfPropertyValue::Text(text) = value else {
        return None;
    };
    let digits = text.strip_prefix('#')?;
    let fits = digits.len() == 6 || (takes_alpha && digits.len() == 8);

    if !fits || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }

    let srgba_u8 = TySrgbaU8::parse_hex(text).expect("the checked digits parse");
    Some(lin_srgba_f64_from_srgba_u8(srgba_u8))
}

/// The number in `value` for the named property `name`, inside the property's
/// range.
fn scalar(check: ArgumentCheck, name: &str, value: &SdfPropertyValue) -> StdResult<f64, String> {
    let range = scalar_range(name).expect("every named number has a range");

    match value {
        SdfPropertyValue::Number(number) if range.contains(*number) => Ok(*number),
        _ => Err(check.fail(name, &expectation(range), shown(value))),
    }
}

/// `range` as an error states it.
fn expectation(range: MaterialRange) -> String {
    let interval = match range.max {
        Some(max) => format!("a number from {} to {max}", range.min),
        None if range.min == 0.0 => "a number of zero or more".to_owned(),
        None => format!("a number of at least {}", range.min),
    };

    if range.admits_zero {
        format!("0 or {interval}")
    } else {
        interval
    }
}

/// The custom property `name` holding `value`.
fn custom_value(
    check: ArgumentCheck,
    name: &str,
    value: &SdfPropertyValue,
) -> StdResult<SdfCustomValue, String> {
    if name.is_empty() {
        return Err("material property name must hold a character, not \"\"".to_owned());
    }

    let vector = |numbers: &Vec<f64>| {
        check.expect(
            (2..=4).contains(&numbers.len()),
            name,
            "a vector of 2 to 4 numbers",
            shown(value),
        )
    };
    let int = |number: f64| {
        check.expect(
            number.fract() == 0.0 && number.abs() <= MOST_INT,
            name,
            "a whole number within 2^53 - 1",
            shown(value),
        )
    };

    Ok(match value {
        SdfPropertyValue::Bool(value) => SdfCustomValue::Bool(*value),

        SdfPropertyValue::Int(number) => {
            int(*number)?;
            SdfCustomValue::Int(*number as i64)
        }

        SdfPropertyValue::IntArray(numbers) => {
            vector(numbers)?;
            numbers.iter().try_for_each(|number| int(*number))?;
            SdfCustomValue::IntVector(numbers.iter().map(|number| *number as i64).collect())
        }

        SdfPropertyValue::Json(value) => SdfCustomValue::Json(value.clone()),

        SdfPropertyValue::Number(number) => SdfCustomValue::Float(*number),

        SdfPropertyValue::NumberArray(numbers) => {
            vector(numbers)?;
            SdfCustomValue::FloatVector(numbers.clone())
        }

        SdfPropertyValue::Text(text) => SdfCustomValue::Text(text.clone()),
    })
}

/// `value` as the TypeScript API writes it.
fn shown(value: &SdfPropertyValue) -> String {
    let list = |numbers: &[f64]| {
        let numbers: Vec<String> = numbers.iter().map(f64::to_string).collect();
        format!("[{}]", numbers.join(", "))
    };

    match value {
        SdfPropertyValue::Bool(value) => value.to_string(),
        SdfPropertyValue::Int(number) => format!("int({number})"),
        SdfPropertyValue::IntArray(numbers) => format!("int({})", list(numbers)),
        SdfPropertyValue::Json(value) => format!("json({})", shown_json(value)),
        SdfPropertyValue::Number(number) => number.to_string(),
        SdfPropertyValue::NumberArray(numbers) => list(numbers),
        SdfPropertyValue::Text(text) => format!("{text:?}"),
    }
}

/// `value` as JSON text.
fn shown_json(value: &SdfValue) -> String {
    match value {
        SdfValue::Array(values) => {
            let values: Vec<String> = values.iter().map(shown_json).collect();
            format!("[{}]", values.join(", "))
        }

        SdfValue::Bool(value) => value.to_string(),

        SdfValue::Null => "null".to_owned(),

        SdfValue::Number(number) => number.to_string(),

        SdfValue::Object(map) => {
            let entries: Vec<String> = map
                .entries()
                .iter()
                .map(|entry| format!("{:?}: {}", entry.key, shown_json(&entry.value)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }

        SdfValue::Text(text) => format!("{text:?}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{SdfCustomValue, material_properties};
    use sdfcore::{SdfProperty, SdfPropertyValue};
    use ty_math::{TyLinSrgbF64, TyLinSrgbaF64};

    /// The property `name` holding `value`.
    fn property(name: &str, value: SdfPropertyValue) -> SdfProperty {
        SdfProperty {
            name: name.to_owned(),
            value,
        }
    }

    /// The error the properties `properties` read.
    fn error(properties: &[SdfProperty]) -> String {
        material_properties(properties).unwrap_err()
    }

    #[test]
    fn a_property_left_out_takes_its_default_and_metallic_takes_zero() {
        let material = material_properties(&[]).unwrap();

        assert_eq!(material.base_color, TyLinSrgbaF64::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(material.emissive_color, TyLinSrgbF64::new(0.0, 0.0, 0.0));
        assert_eq!(
            [
                material.emissive_strength,
                material.ior,
                material.metallic,
                material.occlusion_strength,
                material.roughness,
                material.transmission,
            ],
            [1.0, 1.5, 0.0, 1.0, 1.0, 0.0]
        );
        assert!(material.custom.is_empty());
    }

    #[test]
    fn a_color_decodes_into_linear_light_and_its_alpha_stays_straight() {
        let material = material_properties(&[
            property("baseColor", SdfPropertyValue::Text("#FF000080".to_owned())),
            property(
                "emissiveColor",
                SdfPropertyValue::Text("#00ff00".to_owned()),
            ),
        ])
        .unwrap();

        assert_eq!(
            material.base_color,
            TyLinSrgbaF64::new(1.0, 0.0, 0.0, 128.0 / 255.0)
        );
        assert_eq!(material.emissive_color, TyLinSrgbF64::new(0.0, 1.0, 0.0));

        let mid = material_properties(&[property(
            "baseColor",
            SdfPropertyValue::Text("#808080".to_owned()),
        )])
        .unwrap();
        assert!((mid.base_color.red - 0.215_860_5).abs() < 1e-6);
    }

    #[test]
    fn a_color_parses_only_as_hex_after_a_hash() {
        for (name, text, message) in [
            (
                "baseColor",
                "FF0000",
                "material baseColor must be #RRGGBB or #RRGGBBAA, not \"FF0000\"",
            ),
            (
                "baseColor",
                "#+F+F+F",
                "material baseColor must be #RRGGBB or #RRGGBBAA, not \"#+F+F+F\"",
            ),
            (
                "emissiveColor",
                "#FF000080",
                "material emissiveColor must be #RRGGBB, not \"#FF000080\"",
            ),
        ] {
            assert_eq!(
                error(&[property(name, SdfPropertyValue::Text(text.to_owned()))]),
                message
            );
        }

        assert_eq!(
            error(&[property("baseColor", SdfPropertyValue::Number(1.0))]),
            "material baseColor must be #RRGGBB or #RRGGBBAA, not 1"
        );
    }

    #[test]
    fn a_named_number_falls_in_its_range() {
        for (name, value, message) in [
            (
                "metallic",
                1.5,
                "material metallic must be a number from 0 to 1, not 1.5",
            ),
            (
                "emissiveStrength",
                -1.0,
                "material emissiveStrength must be a number of zero or more, not -1",
            ),
            (
                "ior",
                0.5,
                "material ior must be 0 or a number of at least 1, not 0.5",
            ),
        ] {
            assert_eq!(
                error(&[property(name, SdfPropertyValue::Number(value))]),
                message
            );
        }

        assert_eq!(
            material_properties(&[property("ior", SdfPropertyValue::Number(0.0))])
                .unwrap()
                .ior,
            0.0
        );
        assert_eq!(
            error(&[property("roughness", SdfPropertyValue::Bool(true))]),
            "material roughness must be a number from 0 to 1, not true"
        );
    }

    #[test]
    fn a_custom_property_takes_the_kind_its_value_sets() {
        let material = material_properties(&[
            property("locked", SdfPropertyValue::Bool(true)),
            property("lootTier", SdfPropertyValue::Int(3.0)),
            property("offset", SdfPropertyValue::NumberArray(vec![0.5, 1.0])),
            property("tag", SdfPropertyValue::Text("chest".to_owned())),
        ])
        .unwrap();

        let kinds: Vec<(&str, String)> = material
            .custom
            .iter()
            .map(|(name, value)| (name.as_str(), value.kind()))
            .collect();
        assert_eq!(
            kinds,
            [
                ("locked", "bool".to_owned()),
                ("lootTier", "int".to_owned()),
                ("offset", "vec-2-float".to_owned()),
                ("tag", "string".to_owned()),
            ]
        );
        assert_eq!(material.custom["lootTier"], SdfCustomValue::Int(3));
    }

    #[test]
    fn a_custom_property_needs_a_name_and_a_kind() {
        assert_eq!(
            error(&[property("", SdfPropertyValue::Bool(true))]),
            "material property name must hold a character, not \"\""
        );
        assert_eq!(
            error(&[property("size", SdfPropertyValue::NumberArray(vec![1.0]))]),
            "material size must be a vector of 2 to 4 numbers, not [1]"
        );
        assert_eq!(
            error(&[property("tier", SdfPropertyValue::Int(1.5))]),
            "material tier must be a whole number within 2^53 - 1, not int(1.5)"
        );
        assert_eq!(
            error(&[property(
                "tiers",
                SdfPropertyValue::IntArray(vec![1.0, 2.0f64.powi(53)])
            )]),
            "material tiers must be a whole number within 2^53 - 1, not int([1, 9007199254740992])"
        );
    }
}
