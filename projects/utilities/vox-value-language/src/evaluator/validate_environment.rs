use crate::{CheckedProgram, Components, Error, Result, ValueEnvironment, evaluator::Lengths};

/// Checks the values against the program's types and the groupings'
/// lengths, answering those lengths.
pub(crate) fn validate_environment(
    program: &CheckedProgram,
    environment: &ValueEnvironment,
) -> Result<Lengths> {
    let lengths = Lengths::from_groupings(&environment.groupings)?;

    for (name, expected) in &program.environment.types {
        let value = environment
            .values
            .get(name)
            .ok_or_else(|| Error::MissingValue { name: name.clone() })?;
        let found = value.to_type();

        if found != *expected {
            return Err(Error::ValueType {
                name: name.clone(),
                expected: *expected,
                found,
            });
        }

        let entries = lengths.of(expected.domain);

        if value.entries() != entries {
            return Err(Error::EntryCount {
                name: name.clone(),
                expected: entries,
                found: value.entries(),
            });
        }

        if let Components::F32(components) = value.components()
            && components.iter().any(|component| !component.is_finite())
        {
            return Err(Error::NonFiniteInput { name: name.clone() });
        }
    }

    if let Some(name) = environment
        .values
        .keys()
        .find(|name| !program.environment.types.contains_key(*name))
    {
        return Err(Error::UnexpectedValue { name: name.clone() });
    }

    Ok(lengths)
}
