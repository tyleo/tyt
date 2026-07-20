use crate::{BASE_COLOR_FACTOR, ObjectPropertyRef, resolve_object_property_ref};
use voxcore::{VoxMain, VoxObject};

/// The winning supplier of `baseColorFactor` for `object`, or `None` when
/// no layer supplies it.
pub fn base_color_factor_ref(state: &VoxMain, object: &VoxObject) -> Option<ObjectPropertyRef> {
    resolve_object_property_ref(state, object, BASE_COLOR_FACTOR)
}
