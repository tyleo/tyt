use crate::{
    Error, Result,
    operations::sdf_doc::{
        EntryPaths, SdfCustomValue, SdfMaterialProperties, check_shades, material_properties,
        shade_properties,
    },
};
use branded_id::{IdVec, IteratorExt};
use sdfcore::{BSdfMaterial, SdfMaterial, SdfState};
use std::collections::BTreeMap;

/// The properties of every material in `state`. A custom property takes one
/// kind across the materials, and a material that leaves it out takes the
/// kind's empty value. An error starts with the path of the first step in
/// `paths` that reaches the failing entry.
pub fn resolve_materials(
    state: &SdfState,
    paths: &EntryPaths,
) -> Result<IdVec<BSdfMaterial, SdfMaterialProperties>> {
    let at = |path: String| move |message: String| Error::invalid(format!("{path}: {message}"));

    for (shades_id, shades) in state.shades.iter().enumerate_ids() {
        check_shades(shades).map_err(at(paths.shades(shades_id)))?;
    }

    let mut materials: IdVec<BSdfMaterial, SdfMaterialProperties> =
        IdVec::with_capacity(state.materials.len());
    let mut kinds: BTreeMap<String, SdfCustomValue> = BTreeMap::new();

    for (material_id, material) in state.materials.iter().enumerate_ids() {
        let at_material = || at(paths.material(material_id));

        let properties = match material {
            SdfMaterial::Material { properties } => {
                material_properties(properties).map_err(at_material())?
            }

            SdfMaterial::Shade { shades_id, index } => {
                let shades = &state.shades[shades_id.to_usize_id()];
                shade_properties(&materials[shades.base_id.to_usize_id()], shades, *index)
                    .map_err(at_material())?
            }
        };

        for (name, value) in &properties.custom {
            match kinds.get(name) {
                Some(first) if first.kind() != value.kind() => {
                    return Err(at_material()(format!(
                        "material {name} must be {}, the kind an earlier material gives it, \
                         not {}",
                        first.kind(),
                        value.kind()
                    )));
                }

                Some(_) => {}

                None => {
                    kinds.insert(name.clone(), value.clone());
                }
            }
        }

        materials.push(properties);
    }

    for properties in materials.iter_mut() {
        for (name, first) in &kinds {
            properties
                .custom
                .entry(name.clone())
                .or_insert_with(|| first.empty());
        }
    }

    Ok(materials)
}

#[cfg(test)]
mod tests {
    use crate::{
        Result,
        operations::sdf_doc::{
            EntryPaths, SdfCustomValue, collect_places, resolve_materials, single_part_main,
        },
    };
    use branded_id::{IdVec, U32Id};
    use sdfcore::{
        SdfMain, SdfMaterial, SdfProperty, SdfPropertyValue, SdfShades, SdfShape3d, SdfStep,
        SdfStepMaterial,
    };
    use std::collections::BTreeMap;
    use ty_math::TyVector3F64;

    /// A model whose one step adds a unit sphere of the first of `materials`.
    fn main_of(materials: Vec<SdfMaterial>, shades: Vec<SdfShades>) -> SdfMain {
        let main = single_part_main(
            vec![SdfShape3d::Sphere {
                center: TyVector3F64::ZERO,
                radius: 1.0,
            }],
            vec![SdfStep::Add {
                name: "body".to_owned(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            }],
        );
        let mut state = main.state().clone();
        state.materials = IdVec::from_vec(materials);
        state.shades = IdVec::from_vec(shades);
        SdfMain::new(state).unwrap()
    }

    /// A `material` entry holding `properties`.
    fn material(properties: &[(&str, SdfPropertyValue)]) -> SdfMaterial {
        SdfMaterial::Material {
            properties: properties
                .iter()
                .map(|(name, value)| SdfProperty {
                    name: (*name).to_owned(),
                    value: value.clone(),
                })
                .collect(),
        }
    }

    /// The custom properties of each material of `main`, or the error
    /// resolving the materials reads.
    fn resolved(main: &SdfMain) -> Result<Vec<BTreeMap<String, SdfCustomValue>>> {
        let places = collect_places(main.state());
        let materials = resolve_materials(main.state(), &EntryPaths::new(main.state(), &places))?;
        Ok(materials
            .iter()
            .map(|material| material.custom.clone())
            .collect())
    }

    #[test]
    fn a_custom_property_fills_in_with_its_kinds_empty_value() {
        let main = main_of(
            vec![
                material(&[("lootTier", SdfPropertyValue::Int(3.0))]),
                material(&[("locked", SdfPropertyValue::Bool(true))]),
            ],
            Vec::new(),
        );
        let custom: Vec<Vec<(String, SdfCustomValue)>> = resolved(&main)
            .unwrap()
            .into_iter()
            .map(|custom| custom.into_iter().collect())
            .collect();

        assert_eq!(
            custom[0],
            [
                ("locked".to_owned(), SdfCustomValue::Bool(false)),
                ("lootTier".to_owned(), SdfCustomValue::Int(3)),
            ]
        );
        assert_eq!(
            custom[1],
            [
                ("locked".to_owned(), SdfCustomValue::Bool(true)),
                ("lootTier".to_owned(), SdfCustomValue::Int(0)),
            ]
        );
    }

    #[test]
    fn a_custom_property_takes_one_kind_across_the_materials() {
        let main = main_of(
            vec![
                material(&[("tier", SdfPropertyValue::Int(3.0))]),
                material(&[("tier", SdfPropertyValue::Number(3.0))]),
            ],
            Vec::new(),
        );

        assert_eq!(
            resolved(&main).unwrap_err().to_string(),
            "materials[1]: material tier must be int, the kind an earlier material gives it, \
             not float"
        );
    }

    #[test]
    fn a_shade_reads_its_base_and_an_error_reports_the_reaching_step() {
        let main = main_of(
            vec![
                material(&[("baseColor", SdfPropertyValue::Text("#000000".to_owned()))]),
                SdfMaterial::Shade {
                    shades_id: U32Id::from_u32(0),
                    index: 0,
                },
            ],
            vec![SdfShades {
                base_id: U32Id::from_u32(0),
                count: 3.0,
                spread: None,
            }],
        );
        let error = resolved(&main).unwrap_err().to_string();
        assert!(
            error.starts_with("materials[1]: shades shade 0 must be between black and white"),
            "{error}"
        );

        let reached = main_of(
            vec![material(&[("metallic", SdfPropertyValue::Number(2.0))])],
            Vec::new(),
        );
        assert_eq!(
            resolved(&reached).unwrap_err().to_string(),
            "model/body: material metallic must be a number from 0 to 1, not 2"
        );
    }
}
