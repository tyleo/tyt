use crate::{
    Error, Result,
    operations::mesh::{MeshElement, Transfer, encode_components},
};
use meshdoc::MeshPropertyValue;
use vox_value_language::{Components, Scalar, Value};

/// The property `value` lands as under `transfer`. Only f32 has rows, so an
/// array of a wider bool, integer, or string errors.
pub(crate) fn extra_property_value(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
) -> Result<MeshPropertyValue> {
    let kind = value.to_type();

    if transfer == Transfer::Srgb && value.scalar() != Scalar::F32 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {kind}, and `srgb` transfers f32 alone"),
        ));
    }

    let width = value.dimension().width();
    let array = value.domain().is_array();
    let single = !array && width == 1;

    let flat = || {
        if array && width > 1 {
            Err(Error::mesh_record(
                element.clone(),
                format!("is a {kind}, and an extras entry holds rows of f32 alone"),
            ))
        } else {
            Ok(())
        }
    };

    let ints = |components: Vec<i64>| {
        if single {
            MeshPropertyValue::Int(components[0])
        } else {
            MeshPropertyValue::Ints(components)
        }
    };

    Ok(match value.components() {
        Components::Bool(components) => {
            flat()?;

            if single {
                MeshPropertyValue::Bool(components[0])
            } else {
                MeshPropertyValue::Bools(components.clone())
            }
        }

        Components::F32(components) => {
            let rows = components
                .chunks_exact(width)
                .map(|entry| {
                    Ok(encode_components(element, entry, transfer, false)?
                        .into_iter()
                        .map(narrow)
                        .collect::<Vec<f64>>())
                })
                .collect::<Result<Vec<_>>>()?;

            if single {
                MeshPropertyValue::Float(rows[0][0])
            } else if !array {
                MeshPropertyValue::Floats(rows[0].clone())
            } else if width == 1 {
                MeshPropertyValue::Floats(rows.into_iter().map(|row| row[0]).collect())
            } else {
                MeshPropertyValue::FloatRows(rows)
            }
        }

        Components::String(components) => {
            flat()?;

            if single {
                MeshPropertyValue::Text(components[0].clone())
            } else {
                MeshPropertyValue::Texts(components.clone())
            }
        }

        Components::U8(components) => {
            flat()?;
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U16(components) => {
            flat()?;
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U32(components) => {
            flat()?;
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }
    })
}

/// `component` narrowed to f32 and widened back through its shortest
/// decimal, so a property prints the f32's digits.
fn narrow(component: f64) -> f64 {
    (component as f32)
        .to_string()
        .parse()
        .expect("a finite float's text parses")
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{MeshElement, Transfer, extra_property_value};
    use meshdoc::MeshPropertyValue;
    use vox_value_language::{Components, Dimension, Domain, Value};

    fn element() -> MeshElement {
        MeshElement::MeshExtra {
            name: "bar".to_owned(),
        }
    }

    fn landed(domain: Domain, dimension: Dimension, components: Components) -> MeshPropertyValue {
        let value = Value::new(domain, dimension, components).unwrap();
        extra_property_value(&element(), &value, Transfer::Linear).unwrap()
    }

    #[test]
    fn a_plain_vec1_lands_as_a_leaf_and_wider_shapes_as_lists_and_rows() {
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::F32(vec![0.4])),
            MeshPropertyValue::Float(0.4)
        );
        assert_eq!(
            landed(
                Domain::Plain,
                Dimension::Vec3,
                Components::F32(vec![1.0, 0.5, 0.0])
            ),
            MeshPropertyValue::Floats(vec![1.0, 0.5, 0.0])
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec1,
                Components::F32(vec![1.0, 0.0])
            ),
            MeshPropertyValue::Floats(vec![1.0, 0.0])
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec2,
                Components::F32(vec![1.0, 0.0, 0.5, 0.5])
            ),
            MeshPropertyValue::FloatRows(vec![vec![1.0, 0.0], vec![0.5, 0.5]])
        );
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])),
            MeshPropertyValue::Bool(true)
        );
        assert_eq!(
            landed(
                Domain::Face,
                Dimension::Vec1,
                Components::Bool(vec![true, false])
            ),
            MeshPropertyValue::Bools(vec![true, false])
        );
        assert_eq!(
            landed(Domain::Plain, Dimension::Vec1, Components::U32(vec![7])),
            MeshPropertyValue::Int(7)
        );
        assert_eq!(
            landed(Domain::Swatch, Dimension::Vec1, Components::U8(vec![1, 2])),
            MeshPropertyValue::Ints(vec![1, 2])
        );
        assert_eq!(
            landed(
                Domain::Plain,
                Dimension::Vec1,
                Components::String(vec!["steel".to_owned()])
            ),
            MeshPropertyValue::Text("steel".to_owned())
        );
        assert_eq!(
            landed(
                Domain::Swatch,
                Dimension::Vec1,
                Components::String(vec!["glass".to_owned(), "steel".to_owned()])
            ),
            MeshPropertyValue::Texts(vec!["glass".to_owned(), "steel".to_owned()])
        );
    }

    #[test]
    fn srgb_curves_the_floats_and_an_array_of_wide_non_floats_errors() {
        let grey = Value::new(Domain::Plain, Dimension::Vec1, Components::F32(vec![0.5])).unwrap();
        let MeshPropertyValue::Float(curved) =
            extra_property_value(&element(), &grey, Transfer::Srgb).unwrap()
        else {
            panic!("a plain vec1 lands as a float");
        };
        assert!((curved - 0.7354).abs() < 1e-4);

        let flag =
            Value::new(Domain::Plain, Dimension::Vec1, Components::Bool(vec![true])).unwrap();
        assert!(extra_property_value(&element(), &flag, Transfer::Srgb).is_err());

        let pairs = Value::new(
            Domain::Swatch,
            Dimension::Vec2,
            Components::U8(vec![1, 2, 3, 4]),
        )
        .unwrap();
        assert!(extra_property_value(&element(), &pairs, Transfer::Linear).is_err());
    }
}
