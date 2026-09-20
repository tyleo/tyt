use crate::{Components, evaluator::EntryPairTransform};

/// Applies a merge to two component lists of one type.
pub(crate) fn transform_entry_pairs(
    first: &Components,
    second: &Components,
    transform: &impl EntryPairTransform,
) -> Components {
    match (first, second) {
        (Components::F32(first), Components::F32(second)) => {
            Components::F32(transform.apply(first, second))
        }
        (Components::U8(first), Components::U8(second)) => {
            Components::U8(transform.apply(first, second))
        }
        (Components::U16(first), Components::U16(second)) => {
            Components::U16(transform.apply(first, second))
        }
        (Components::U32(first), Components::U32(second)) => {
            Components::U32(transform.apply(first, second))
        }
        (Components::Bool(first), Components::Bool(second)) => {
            Components::Bool(transform.apply(first, second))
        }
        (Components::String(first), Components::String(second)) => {
            Components::String(transform.apply(first, second))
        }
        _ => unreachable!("the checker settles one type across the pair"),
    }
}
