use crate::{Components, Dimension, Domain, Value};

/// A vec1 value of bool components.
pub fn bools(domain: Domain, values: &[bool]) -> Value {
    Value::new(domain, Dimension::Vec1, Components::Bool(values.to_vec())).unwrap()
}
