use crate::{
    Result,
    operations::sdf_doc::{SdfCustomValue, SdfMaterialProperties},
};
use branded_id::{IdVec, U32Id};
use sdfcore::{BSdfMaterial, SdfValue};
use std::collections::HashMap;
use voxcore::{
    BVoxMaterial, BVoxPalette, BVoxValuePoolValue, VoxMain, VoxMap, VoxMapEntry, VoxPalette,
    VoxValue, VoxValuePool,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, OCCLUSION_STRENGTH,
        ROUGHNESS, TRANSMISSION,
    },
};

/// A palette's id with the palette material each model material takes.
type SdfPalette = (
    U32Id<BVoxPalette>,
    HashMap<U32Id<BSdfMaterial>, U32Id<BVoxMaterial>>,
);

/// A property's value pool with the value id each palette material takes.
type PropertyColumn = (VoxValuePool, Vec<U32Id<BVoxValuePoolValue>>);

/// Adds the palette of the materials at `material_ids` to `main`. The palette's
/// materials follow `material_ids`, and materials with identical properties
/// merge into one.
pub fn retain_sdf_palette(
    main: &mut VoxMain,
    materials: &IdVec<BSdfMaterial, SdfMaterialProperties>,
    material_ids: &[U32Id<BSdfMaterial>],
) -> Result<SdfPalette> {
    let mut distinct: Vec<&SdfMaterialProperties> = Vec::new();
    let distinct_indices: Vec<usize> = material_ids
        .iter()
        .map(|material_id| {
            let properties = &materials[material_id.to_usize_id()];

            distinct
                .iter()
                .position(|known| *known == properties)
                .unwrap_or_else(|| {
                    distinct.push(properties);
                    distinct.len() - 1
                })
        })
        .collect();

    let float = |get: fn(&SdfMaterialProperties) -> f64| -> Result<PropertyColumn> {
        let (values, value_ids) = column(distinct.iter().map(|properties| get(properties)));
        Ok((VoxValuePool::float(values)?, value_ids))
    };

    let (base_colors, base_color_ids) = column(
        distinct
            .iter()
            .map(|properties| <[f64; 4]>::from(properties.base_color)),
    );
    let (emissive_colors, emissive_color_ids) = column(
        distinct
            .iter()
            .map(|properties| <[f64; 3]>::from(properties.emissive_color)),
    );

    let mut properties: Vec<(&str, PropertyColumn)> = vec![
        (
            BASE_COLOR,
            (VoxValuePool::vec_4_float(base_colors)?, base_color_ids),
        ),
        (METALLIC, float(|properties| properties.metallic)?),
        (ROUGHNESS, float(|properties| properties.roughness)?),
        (
            EMISSIVE_COLOR,
            (
                VoxValuePool::vec_3_float(emissive_colors)?,
                emissive_color_ids,
            ),
        ),
        (
            EMISSIVE_STRENGTH,
            float(|properties| properties.emissive_strength)?,
        ),
        (
            OCCLUSION_STRENGTH,
            float(|properties| properties.occlusion_strength)?,
        ),
        (IOR, float(|properties| properties.ior)?),
        (TRANSMISSION, float(|properties| properties.transmission)?),
    ];

    let custom_names = distinct.first().map_or(Vec::new(), |properties| {
        properties.custom.keys().map(String::as_str).collect()
    });

    for name in custom_names {
        let (values, value_ids) = column(
            distinct
                .iter()
                .map(|properties| properties.custom[name].clone()),
        );
        properties.push((name, (custom_value_pool(values)?, value_ids)));
    }

    let mut palette = VoxPalette::default();
    let mut columns = Vec::with_capacity(properties.len());

    for (name, (value_pool, value_ids)) in properties {
        let value_pool_id = main.retain_value_pool(value_pool);
        palette
            .retain_property(name.to_owned(), value_pool_id)
            .expect("the property names are distinct");
        columns.push(value_ids);
    }

    let palette_material_ids: Vec<U32Id<BVoxMaterial>> = (0..distinct.len())
        .map(|index| {
            palette
                .retain_material(columns.iter().map(|value_ids| value_ids[index]).collect())
                .expect("one value id for each property")
        })
        .collect();

    let palette_id = main.retain_palette(palette)?;

    Ok((
        palette_id,
        material_ids
            .iter()
            .zip(distinct_indices)
            .map(|(material_id, index)| (*material_id, palette_material_ids[index]))
            .collect(),
    ))
}

/// The distinct values of `values` in first-seen order, with the value id each
/// value takes.
fn column<T: PartialEq>(
    values: impl Iterator<Item = T>,
) -> (Vec<T>, Vec<U32Id<BVoxValuePoolValue>>) {
    let mut distinct = Vec::new();

    let value_ids = values
        .map(|value| {
            let index = distinct
                .iter()
                .position(|known| *known == value)
                .unwrap_or_else(|| {
                    distinct.push(value);
                    distinct.len() - 1
                });

            U32Id::from_u32(index as u32)
        })
        .collect();

    (distinct, value_ids)
}

/// The value pool of one custom property's distinct `values`, which share a
/// kind.
fn custom_value_pool(values: Vec<SdfCustomValue>) -> Result<VoxValuePool> {
    let first = values.first().expect("a palette holds a material").clone();

    Ok(match first {
        SdfCustomValue::Bool(_) => VoxValuePool::boolean(of_kind(values, |value| match value {
            SdfCustomValue::Bool(value) => Some(value),
            _ => None,
        })),

        SdfCustomValue::Float(_) => VoxValuePool::float(of_kind(values, |value| match value {
            SdfCustomValue::Float(value) => Some(value),
            _ => None,
        }))?,

        SdfCustomValue::FloatVector(numbers) => {
            let vectors = of_kind(values, |value| match value {
                SdfCustomValue::FloatVector(numbers) => Some(numbers),
                _ => None,
            });

            match numbers.len() {
                2 => VoxValuePool::vec_2_float(arrays(vectors))?,
                3 => VoxValuePool::vec_3_float(arrays(vectors))?,
                _ => VoxValuePool::vec_4_float(arrays(vectors))?,
            }
        }

        SdfCustomValue::Int(_) => VoxValuePool::int(of_kind(values, |value| match value {
            SdfCustomValue::Int(value) => Some(value),
            _ => None,
        }))?,

        SdfCustomValue::IntVector(numbers) => {
            let vectors = of_kind(values, |value| match value {
                SdfCustomValue::IntVector(numbers) => Some(numbers),
                _ => None,
            });

            match numbers.len() {
                2 => VoxValuePool::vec_2_int(arrays(vectors))?,
                3 => VoxValuePool::vec_3_int(arrays(vectors))?,
                _ => VoxValuePool::vec_4_int(arrays(vectors))?,
            }
        }

        SdfCustomValue::Json(_) => VoxValuePool::json(of_kind(values, |value| match value {
            SdfCustomValue::Json(value) => Some(vox_value(&value)),
            _ => None,
        })),

        SdfCustomValue::Text(_) => VoxValuePool::string(of_kind(values, |value| match value {
            SdfCustomValue::Text(value) => Some(value),
            _ => None,
        })),
    })
}

/// What `get` reads from each of `values`. Every value holds the kind `get`
/// reads.
fn of_kind<T>(values: Vec<SdfCustomValue>, get: impl Fn(SdfCustomValue) -> Option<T>) -> Vec<T> {
    values
        .into_iter()
        .map(|value| get(value).expect("a custom property holds one kind"))
        .collect()
}

/// `vectors` as arrays of their shared length.
fn arrays<T, const N: usize>(vectors: Vec<Vec<T>>) -> Vec<[T; N]> {
    vectors
        .into_iter()
        .map(|vector| {
            <[T; N]>::try_from(vector)
                .unwrap_or_else(|_| panic!("a vector property holds one length"))
        })
        .collect()
}

/// A `json` value as voxcore holds it.
fn vox_value(value: &SdfValue) -> VoxValue {
    match value {
        SdfValue::Array(values) => VoxValue::Array(values.iter().map(vox_value).collect()),
        SdfValue::Bool(value) => VoxValue::Bool(*value),
        SdfValue::Null => VoxValue::Null,
        SdfValue::Number(value) => VoxValue::Number(*value),
        SdfValue::Object(map) => VoxValue::Object(VoxMap::new(
            map.entries()
                .iter()
                .map(|entry| VoxMapEntry {
                    key: entry.key.clone(),
                    value: vox_value(&entry.value),
                })
                .collect(),
        )),
        SdfValue::Text(value) => VoxValue::Text(value.clone()),
    }
}
