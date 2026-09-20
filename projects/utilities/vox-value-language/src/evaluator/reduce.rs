use crate::{
    Components, Domain, EvalFailure, Groupings, Value,
    checker::Reduction,
    evaluator::{EvalResult, Lengths, Numeric, groups},
};

/// Reduces an array into a domain below it, each destination entry
/// gathering its source entries per component.
pub(crate) fn reduce(
    value: &Value,
    reduction: Reduction,
    target: Domain,
    groupings: &Groupings,
    lengths: &Lengths,
) -> EvalResult<Value> {
    let groups = groups(value.domain(), target, groupings, lengths);
    let operation = reduction.name(target);
    let width = value.dimension().width();
    let components = match (reduction, value.components()) {
        (Reduction::Avg, Components::F32(components)) => {
            Components::F32(average(components, width, &groups, &operation, target)?)
        }
        (Reduction::Avg, Components::U8(components)) => {
            Components::F32(average(components, width, &groups, &operation, target)?)
        }
        (Reduction::Avg, Components::U16(components)) => {
            Components::F32(average(components, width, &groups, &operation, target)?)
        }
        (Reduction::Avg, Components::U32(components)) => {
            Components::F32(average(components, width, &groups, &operation, target)?)
        }
        (_, Components::F32(components)) => Components::F32(fold(
            components, width, &groups, reduction, &operation, target,
        )?),
        (_, Components::U8(components)) => Components::U8(fold(
            components, width, &groups, reduction, &operation, target,
        )?),
        (_, Components::U16(components)) => Components::U16(fold(
            components, width, &groups, reduction, &operation, target,
        )?),
        (_, Components::U32(components)) => Components::U32(fold(
            components, width, &groups, reduction, &operation, target,
        )?),
        (_, Components::Bool(_) | Components::String(_)) => {
            unreachable!("the checker rejects a reduction over bools and strings")
        }
    };

    Ok(Value::new(target, value.dimension(), components)
        .expect("a reduction fills every entry of the target"))
}

/// The mean of each group per component, accumulated in `f64`.
fn average<T: Numeric>(
    components: &[T],
    width: usize,
    groups: &[Vec<usize>],
    operation: &str,
    target: Domain,
) -> EvalResult<Vec<f32>> {
    let mut output = Vec::with_capacity(groups.len() * width);

    for (entry, group) in groups.iter().enumerate() {
        if group.is_empty() {
            return Err(EvalFailure::EmptyDestination {
                operation: operation.to_owned(),
                target,
                entry,
            });
        }

        for position in 0..width {
            let total = group
                .iter()
                .map(|&source| components[source * width + position].to_f64())
                .sum::<f64>();
            let mean = (total / group.len() as f64) as f32;

            if !mean.is_finite() {
                return Err(EvalFailure::NonFinite {
                    operation: operation.to_owned(),
                });
            }

            output.push(mean);
        }
    }

    Ok(output)
}

/// The min, max, or sum of each group per component.
fn fold<T: Numeric>(
    components: &[T],
    width: usize,
    groups: &[Vec<usize>],
    reduction: Reduction,
    operation: &str,
    target: Domain,
) -> EvalResult<Vec<T>> {
    let mut output = Vec::with_capacity(groups.len() * width);

    for (entry, group) in groups.iter().enumerate() {
        for position in 0..width {
            let mut values = group
                .iter()
                .map(|&source| components[source * width + position]);
            let value = match reduction {
                Reduction::Sum => T::sum(values, operation)?,

                Reduction::Max | Reduction::Min => {
                    let first = values.next().ok_or_else(|| EvalFailure::EmptyDestination {
                        operation: operation.to_owned(),
                        target,
                        entry,
                    })?;

                    values.fold(first, |best, value| match reduction {
                        Reduction::Max if value > best => value,
                        Reduction::Min if value < best => value,
                        _ => best,
                    })
                }

                Reduction::Avg => unreachable!("averages take their own path"),
            };

            output.push(value);
        }
    }

    Ok(output)
}
