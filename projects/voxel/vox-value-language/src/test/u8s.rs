use crate::{Components, Dimension, Domain, Value};

/// A value of `u8` components.
pub fn u8s(domain: Domain, dimension: Dimension, values: &[u8]) -> Value {
    Value::new(domain, dimension, Components::U8(values.to_vec())).unwrap()
}
