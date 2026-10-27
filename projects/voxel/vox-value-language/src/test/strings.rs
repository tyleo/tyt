use crate::{Components, Dimension, Domain, Value};

/// A vec1 value of string components.
pub fn strings(domain: Domain, values: &[&str]) -> Value {
    Value::new(
        domain,
        Dimension::Vec1,
        Components::String(values.iter().map(|value| (*value).to_owned()).collect()),
    )
    .unwrap()
}
