use crate::GltfBlob;
use gltf::json::{
    Accessor, Index, Root,
    accessor::{ComponentType, GenericComponentType, Type},
    buffer::Target,
    validation::{Checked, USize64},
};
use serde_json::Value;

/// Appends `data`, `count` elements of `component_type` by `type_`, to the
/// blob as its own view and an accessor over it, returning the accessor's
/// index.
#[allow(clippy::too_many_arguments)]
pub fn push_accessor(
    root: &mut Root,
    blob: &mut GltfBlob,
    data: &[u8],
    count: usize,
    component_type: ComponentType,
    type_: Type,
    normalized: bool,
    target: Option<Target>,
    min: Option<Value>,
    max: Option<Value>,
) -> Index<Accessor> {
    let buffer_view = blob.push_view(data, target);

    let accessor = Accessor {
        buffer_view: Some(buffer_view),
        byte_offset: None,
        count: USize64::from(count),
        component_type: Checked::Valid(GenericComponentType(component_type)),
        extensions: None,
        extras: Default::default(),
        type_: Checked::Valid(type_),
        min,
        max,
        name: None,
        normalized,
        sparse: None,
    };

    root.push(accessor)
}
