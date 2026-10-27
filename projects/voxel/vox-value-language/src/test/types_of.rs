use crate::{TypeEnvironment, ValueEnvironment};

/// The type environment every value in the environment implies.
pub fn types_of(environment: &ValueEnvironment) -> TypeEnvironment {
    TypeEnvironment {
        types: environment
            .values
            .iter()
            .map(|(name, value)| (name.clone(), value.to_type()))
            .collect(),
    }
}
