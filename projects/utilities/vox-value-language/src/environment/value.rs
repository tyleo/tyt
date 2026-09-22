use crate::{Components, Dimension, Domain, Error, Result, Scalar, Type};

/// A value: a plain entry or one entry per element of its domain, each entry
/// `dimension` components wide.
#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    domain: Domain,
    dimension: Dimension,
    components: Components,
}

impl Value {
    /// Builds a value, erroring on a component count that disagrees with the
    /// domain and dimension and on a string above vec1.
    ///
    /// # Arguments
    /// - `domain`: what the value has one entry per.
    /// - `dimension`: the vec width.
    /// - `components`: the entries, flattened component by component; a plain
    ///   value holds exactly one entry, and an array any whole number of them.
    pub fn new(domain: Domain, dimension: Dimension, components: Components) -> Result<Value> {
        if components.scalar() == Scalar::String && dimension != Dimension::Vec1 {
            return Err(Error::StringWidth { dimension });
        }

        let width = dimension.width();
        let count = components.len();
        let fits = if domain.is_array() {
            count.is_multiple_of(width)
        } else {
            count == width
        };

        if !fits {
            return Err(Error::ComponentCount {
                domain,
                dimension,
                components: count,
            });
        }

        Ok(Value {
            domain,
            dimension,
            components,
        })
    }

    /// The entries, flattened component by component.
    pub fn components(&self) -> &Components {
        &self.components
    }

    /// The vec width.
    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    /// What the value has one entry per.
    pub fn domain(&self) -> Domain {
        self.domain
    }

    /// The entry count: one for a plain value, the domain's length for an
    /// array.
    pub fn entries(&self) -> usize {
        self.components.len() / self.dimension.width()
    }

    /// The component type.
    pub fn scalar(&self) -> Scalar {
        self.components.scalar()
    }

    /// The value's type.
    pub fn to_type(&self) -> Type {
        Type {
            domain: self.domain,
            dimension: self.dimension,
            scalar: self.scalar(),
        }
    }

    /// Unwraps into the flattened components.
    pub fn into_components(self) -> Components {
        self.components
    }
}

#[cfg(test)]
mod tests {
    use crate::{Components, Dimension, Domain, Error, Scalar, Type, Value};

    #[test]
    fn a_plain_value_holds_exactly_one_entry() {
        let value = Value::new(
            Domain::Plain,
            Dimension::Vec3,
            Components::F32(vec![1.0, 2.0, 3.0]),
        )
        .unwrap();

        assert_eq!(value.entries(), 1);
        assert_eq!(
            value.to_type(),
            Type {
                domain: Domain::Plain,
                dimension: Dimension::Vec3,
                scalar: Scalar::F32
            }
        );
        assert_eq!(
            Value::new(
                Domain::Plain,
                Dimension::Vec3,
                Components::F32(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
            ),
            Err(Error::ComponentCount {
                domain: Domain::Plain,
                dimension: Dimension::Vec3,
                components: 6
            })
        );
    }

    #[test]
    fn an_array_holds_a_whole_number_of_entries() {
        let value = Value::new(
            Domain::Swatch,
            Dimension::Vec2,
            Components::U8(vec![1, 2, 3, 4, 5, 6]),
        )
        .unwrap();

        assert_eq!(value.entries(), 3);
        assert_eq!(value.scalar(), Scalar::U8);
        assert!(Value::new(Domain::Face, Dimension::Vec4, Components::U32(vec![])).is_ok());
        assert_eq!(
            Value::new(
                Domain::Swatch,
                Dimension::Vec2,
                Components::U8(vec![1, 2, 3])
            ),
            Err(Error::ComponentCount {
                domain: Domain::Swatch,
                dimension: Dimension::Vec2,
                components: 3
            })
        );
    }

    #[test]
    fn a_string_is_vec1_alone() {
        assert!(
            Value::new(
                Domain::Swatch,
                Dimension::Vec2,
                Components::Bool(vec![true, false])
            )
            .is_ok()
        );
        assert_eq!(
            Value::new(
                Domain::Plain,
                Dimension::Vec2,
                Components::String(vec!["a".to_owned(), "b".to_owned()])
            ),
            Err(Error::StringWidth {
                dimension: Dimension::Vec2
            })
        );
    }
}
