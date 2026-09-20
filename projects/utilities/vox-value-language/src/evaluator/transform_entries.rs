use crate::{
    Components,
    evaluator::{EntryTransform, EvalResult},
};

/// Applies a rearrangement to components of any type.
pub(crate) fn transform_entries(
    components: &Components,
    transform: &impl EntryTransform,
) -> EvalResult<Components> {
    Ok(match components {
        Components::F32(components) => Components::F32(transform.apply(components)?),
        Components::U8(components) => Components::U8(transform.apply(components)?),
        Components::U16(components) => Components::U16(transform.apply(components)?),
        Components::U32(components) => Components::U32(transform.apply(components)?),
        Components::Bool(components) => Components::Bool(transform.apply(components)?),
        Components::String(components) => Components::String(transform.apply(components)?),
    })
}
