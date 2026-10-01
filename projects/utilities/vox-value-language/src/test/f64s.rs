use crate::{Components, Dimension, Domain, Value};

/// A value of `f64` components.
pub fn f64s(domain: Domain, dimension: Dimension, values: &[f64]) -> Value {
    Value::new(domain, dimension, Components::F64(values.to_vec())).unwrap()
}
