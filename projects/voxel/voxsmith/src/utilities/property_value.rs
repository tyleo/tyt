use crate::{Error, Result};
use branded_id::U32Id;
use std::slice;
use vox_value_language::{Components, Dimension, Domain, Value};
use voxcore::{BVoxValuePoolValue, VoxValueColumn, VoxValuePool, VoxValuePoolValues};

/// The swatch array of the property `name` over `value_pool`, reading each
/// swatch's value id in `value_ids`, or `None` for a json property, which the
/// language has no type for. An int reads as `u32` and errors outside its
/// range.
pub fn property_value(
    name: &str,
    value_pool: &VoxValuePool,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> Result<Option<Value>> {
    let (dimension, components) = match value_pool.values() {
        VoxValuePoolValues::Bool(flags) => (
            Dimension::Vec1,
            Components::Bool(swatch_values(flags, value_ids).copied().collect()),
        ),

        VoxValuePoolValues::Float(numbers) => (
            Dimension::Vec1,
            Components::F64(swatch_values(numbers, value_ids).copied().collect()),
        ),

        VoxValuePoolValues::Int(numbers) => (
            Dimension::Vec1,
            u32s(
                name,
                swatch_values(numbers, value_ids).map(slice::from_ref),
                1,
            )?,
        ),

        VoxValuePoolValues::Json(_) => return Ok(None),

        VoxValuePoolValues::String(texts) => (
            Dimension::Vec1,
            Components::String(swatch_values(texts, value_ids).cloned().collect()),
        ),

        VoxValuePoolValues::Vec2Float(vectors) => (Dimension::Vec2, f64s(vectors, value_ids)),

        VoxValuePoolValues::Vec2Int(vectors) => (
            Dimension::Vec2,
            u32s(
                name,
                swatch_values(vectors, value_ids).map(|vector| vector.as_slice()),
                2,
            )?,
        ),

        VoxValuePoolValues::Vec3Float(vectors) => (Dimension::Vec3, f64s(vectors, value_ids)),

        VoxValuePoolValues::Vec3Int(vectors) => (
            Dimension::Vec3,
            u32s(
                name,
                swatch_values(vectors, value_ids).map(|vector| vector.as_slice()),
                3,
            )?,
        ),

        VoxValuePoolValues::Vec4Float(vectors) => (Dimension::Vec4, f64s(vectors, value_ids)),

        VoxValuePoolValues::Vec4Int(vectors) => (
            Dimension::Vec4,
            u32s(
                name,
                swatch_values(vectors, value_ids).map(|vector| vector.as_slice()),
                4,
            )?,
        ),
    };

    let value = Value::new(Domain::Swatch, dimension, components)
        .expect("every swatch contributes one entry of the pool's width");

    Ok(Some(value))
}

/// The value each of `value_ids` draws from `values`, in swatch order.
fn swatch_values<'a, T>(
    values: VoxValueColumn<'a, T>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> impl ExactSizeIterator<Item = &'a T> {
    value_ids.iter().map(move |&value_id| {
        values
            .get(value_id)
            .expect("a swatch draws one of its property's values")
    })
}

/// The flattened components of the float vectors `value_ids` draw.
fn f64s<const N: usize>(
    vectors: VoxValueColumn<'_, [f64; N]>,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> Components {
    Components::F64(
        swatch_values(vectors, value_ids)
            .flatten()
            .copied()
            .collect(),
    )
}

/// The int components of `swatch_components` flattened as `u32`. Each swatch
/// holds one slice of `width`. Errors on a component outside the `u32` range
/// and reports its swatch.
fn u32s<'a>(
    name: &str,
    swatch_components: impl ExactSizeIterator<Item = &'a [i64]>,
    width: usize,
) -> Result<Components> {
    let mut components = Vec::with_capacity(swatch_components.len() * width);

    for (swatch, values) in (0..).zip(swatch_components) {
        for &component in values {
            let component = u32::try_from(component).map_err(|_| {
                Error::invalid(format!(
                    "the property `{name}` holds {component} at swatch {swatch}, and the value \
                     language reads an int as u32"
                ))
            })?;

            components.push(component);
        }
    }

    Ok(Components::U32(components))
}

#[cfg(test)]
mod tests {
    use crate::utilities::property_value;
    use vox_value_language::{Components, Dimension, Domain, Scalar};
    use voxcore::{VoxValue, VoxValuePool};

    #[test]
    fn each_pool_kind_binds_its_language_type() {
        let cases: [(VoxValuePool, Dimension, Scalar, usize); 6] = [
            (
                VoxValuePool::boolean(vec![true]),
                Dimension::Vec1,
                Scalar::Bool,
                1,
            ),
            (
                VoxValuePool::float(vec![0.5]).unwrap(),
                Dimension::Vec1,
                Scalar::F64,
                1,
            ),
            (
                VoxValuePool::int(vec![7]).unwrap(),
                Dimension::Vec1,
                Scalar::U32,
                1,
            ),
            (
                VoxValuePool::string(vec!["glass".to_owned()]),
                Dimension::Vec1,
                Scalar::String,
                1,
            ),
            (
                VoxValuePool::vec_3_int(vec![[1, 2, 3]]).unwrap(),
                Dimension::Vec3,
                Scalar::U32,
                3,
            ),
            (
                VoxValuePool::vec_4_float(vec![[0.0, 0.5, 1.0, 1.0]]).unwrap(),
                Dimension::Vec4,
                Scalar::F64,
                4,
            ),
        ];

        for (pool, dimension, scalar, components) in cases {
            let value_ids: Vec<_> = pool.iter_value_ids().collect();

            let value = property_value("p", &pool, &value_ids).unwrap().unwrap();

            assert_eq!(value.domain(), Domain::Swatch, "{dimension} {scalar}");
            assert_eq!(value.dimension(), dimension, "{dimension} {scalar}");
            assert_eq!(value.scalar(), scalar, "{dimension} {scalar}");
            assert_eq!(value.components().len(), components, "{dimension} {scalar}");
        }
    }

    #[test]
    fn swatches_flatten_in_order() {
        let pool = VoxValuePool::vec_2_float(vec![[1.0, 2.0], [3.0, 4.0]]).unwrap();
        let value_ids: Vec<_> = pool.iter_value_ids().collect();

        let value = property_value("p", &pool, &value_ids).unwrap().unwrap();

        assert_eq!(value.entries(), 2);
        assert_eq!(
            value.components(),
            &Components::F64(vec![1.0, 2.0, 3.0, 4.0])
        );
    }

    #[test]
    fn an_int_outside_u32_errors_at_its_swatch() {
        let pool = VoxValuePool::int(vec![1, -1]).unwrap();
        let value_ids: Vec<_> = pool.iter_value_ids().collect();

        let error = property_value("tag", &pool, &value_ids)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`tag` holds -1 at swatch 1"), "{error}");
    }

    #[test]
    fn a_json_property_binds_nothing() {
        let pool = VoxValuePool::json(vec![VoxValue::Null]);
        let value_ids: Vec<_> = pool.iter_value_ids().collect();

        assert_eq!(property_value("meta", &pool, &value_ids).unwrap(), None);
    }
}
