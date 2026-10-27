use crate::{Components, Dimension, Domain, Value};

/// A value of `u32` components.
pub fn u32s(domain: Domain, dimension: Dimension, values: &[u32]) -> Value {
    Value::new(domain, dimension, Components::U32(values.to_vec())).unwrap()
}
