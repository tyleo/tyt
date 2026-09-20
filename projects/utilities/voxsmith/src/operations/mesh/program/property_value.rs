use crate::{Error, Result};
use std::slice;
use vox_value_language::{Components, Dimension, Domain, Value};
use voxcore::{VoxValuePoolKind, VoxValuePoolValueRef};

/// The swatch array of the property `name`, whose pool has `kind`, from its
/// per-swatch `values`, or `None` for a json property, which the language
/// has no type for. An int reads as `u32` and errors outside its range.
pub(crate) fn property_value(
    name: &str,
    kind: &VoxValuePoolKind,
    values: &[VoxValuePoolValueRef<'_>],
) -> Result<Option<Value>> {
    let (dimension, components) = match kind {
        VoxValuePoolKind::Bool(_) => (
            Dimension::Vec1,
            Components::Bool(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::Bool(value) => *value,
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Float(_) => (
            Dimension::Vec1,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Float(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Int(_) => (
            Dimension::Vec1,
            u32s(name, values, 1, |value| match value {
                VoxValuePoolValueRef::Int(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Json(_) => return Ok(None),

        VoxValuePoolKind::String(_) => (
            Dimension::Vec1,
            Components::String(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::String(value) => (*value).to_owned(),
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Vec2Float(_) => (
            Dimension::Vec2,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec2Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec2Int(_) => (
            Dimension::Vec2,
            u32s(name, values, 2, |value| match value {
                VoxValuePoolValueRef::Vec2Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec3Float(_) => (
            Dimension::Vec3,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec3Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec3Int(_) => (
            Dimension::Vec3,
            u32s(name, values, 3, |value| match value {
                VoxValuePoolValueRef::Vec3Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec4Float(_) => (
            Dimension::Vec4,
            f32s(values, |value| match value {
                VoxValuePoolValueRef::Vec4Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec4Int(_) => (
            Dimension::Vec4,
            u32s(name, values, 4, |value| match value {
                VoxValuePoolValueRef::Vec4Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),
    };

    let value = Value::new(Domain::Swatch, dimension, components)
        .expect("every swatch contributes one entry of the pool's width");

    Ok(Some(value))
}

/// The float components of `values` flattened, each read through `pick`.
fn f32s<'a>(
    values: &[VoxValuePoolValueRef<'a>],
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [f64],
) -> Components {
    Components::F32(
        values
            .iter()
            .flat_map(|value| pick(value).iter().map(|&component| component as f32))
            .collect(),
    )
}

/// The int components of `values` flattened as `u32`, each read through
/// `pick`, erroring on one outside the range and reporting its swatch.
fn u32s<'a>(
    name: &str,
    values: &[VoxValuePoolValueRef<'a>],
    width: usize,
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [i64],
) -> Result<Components> {
    let mut components = Vec::with_capacity(values.len() * width);

    for (swatch, value) in (0..).zip(values) {
        for &component in pick(value) {
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
    use crate::operations::mesh::property_value;
    use vox_value_language::{Components, Dimension, Domain, Scalar};
    use voxcore::{VoxValue, VoxValuePool, VoxValuePoolValueRef};

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
                Scalar::F32,
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
                Scalar::F32,
                4,
            ),
        ];

        for (pool, dimension, scalar, components) in cases {
            let values: Vec<VoxValuePoolValueRef> =
                pool.iter_values().map(|(_, value)| value).collect();

            let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

            assert_eq!(value.domain(), Domain::Swatch, "{dimension} {scalar}");
            assert_eq!(value.dimension(), dimension, "{dimension} {scalar}");
            assert_eq!(value.scalar(), scalar, "{dimension} {scalar}");
            assert_eq!(value.components().len(), components, "{dimension} {scalar}");
        }
    }

    #[test]
    fn swatches_flatten_in_order() {
        let pool = VoxValuePool::vec_2_float(vec![[1.0, 2.0], [3.0, 4.0]]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

        assert_eq!(value.entries(), 2);
        assert_eq!(
            value.components(),
            &Components::F32(vec![1.0, 2.0, 3.0, 4.0])
        );
    }

    #[test]
    fn an_int_outside_u32_errors_at_its_swatch() {
        let pool = VoxValuePool::int(vec![1, -1]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let error = property_value("tag", pool.kind(), &values)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`tag` holds -1 at swatch 1"), "{error}");
    }

    #[test]
    fn a_json_property_binds_nothing() {
        let pool = VoxValuePool::json(vec![VoxValue::Null]);
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        assert_eq!(property_value("meta", pool.kind(), &values).unwrap(), None);
    }
}
