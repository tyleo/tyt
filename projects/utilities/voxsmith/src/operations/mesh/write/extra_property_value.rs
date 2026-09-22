use crate::{
    Error, Result,
    operations::mesh::{MeshElement, Transfer, encode_components},
};
use meshdoc::MeshPropertyValue;
use vox_value_language::{Components, Scalar, Value};

/// The property `value` lands as under `transfer`.
pub(crate) fn extra_property_value(
    element: &MeshElement,
    value: &Value,
    transfer: Transfer,
) -> Result<MeshPropertyValue> {
    if transfer == Transfer::Srgb && value.scalar() != Scalar::F32 {
        return Err(Error::mesh_record(
            element.clone(),
            format!("is a {}, and `srgb` transfers f32 alone", value.to_type()),
        ));
    }

    let shape = Shape {
        width: value.dimension().width(),
        array: value.domain().is_array(),
    };

    let ints = |components: Vec<i64>| {
        shape.land(
            &components,
            MeshPropertyValue::Int,
            MeshPropertyValue::Ints,
            MeshPropertyValue::IntRows,
        )
    };

    Ok(match value.components() {
        Components::Bool(components) => shape.land(
            components,
            MeshPropertyValue::Bool,
            MeshPropertyValue::Bools,
            MeshPropertyValue::BoolRows,
        ),

        Components::F32(components) => {
            let encoded: Vec<f64> = components
                .chunks_exact(shape.width)
                .map(|entry| encode_components(element, entry, transfer, false))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .map(narrow)
                .collect();

            shape.land(
                &encoded,
                MeshPropertyValue::Float,
                MeshPropertyValue::Floats,
                MeshPropertyValue::FloatRows,
            )
        }

        Components::String(components) => shape.land(
            components,
            MeshPropertyValue::Text,
            MeshPropertyValue::Texts,
            MeshPropertyValue::TextRows,
        ),

        Components::U8(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U16(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }

        Components::U32(components) => {
            ints(components.iter().map(|&number| i64::from(number)).collect())
        }
    })
}

/// The shape a value's components land in.
#[derive(Clone, Copy)]
struct Shape {
    width: usize,
    array: bool,
}

impl Shape {
    /// `components` as a leaf for a plain vec1, a list for a plain vecN or a
    /// vec1 array, or rows of `width` for a vecN array.
    fn land<T: Clone>(
        self,
        components: &[T],
        leaf: fn(T) -> MeshPropertyValue,
        list: fn(Vec<T>) -> MeshPropertyValue,
        rows: fn(Vec<Vec<T>>) -> MeshPropertyValue,
    ) -> MeshPropertyValue {
        if !self.array && self.width == 1 {
            leaf(components[0].clone())
        } else if !self.array || self.width == 1 {
            list(components.to_vec())
        } else {
            rows(
                components
                    .chunks_exact(self.width)
                    .map(<[T]>::to_vec)
                    .collect(),
            )
        }
    }
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
            landed(
                Domain::Face,
                Dimension::Vec2,
                Components::Bool(vec![true, false, false, true])
            ),
            MeshPropertyValue::BoolRows(vec![vec![true, false], vec![false, true]])
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
                Domain::Swatch,
                Dimension::Vec2,
                Components::U8(vec![1, 2, 3, 4])
            ),
            MeshPropertyValue::IntRows(vec![vec![1, 2], vec![3, 4]])
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
    fn srgb_curves_the_floats_and_errors_on_a_non_float() {
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
    }
}
