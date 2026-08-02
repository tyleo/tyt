use crate::voxj_value_from_vox_value;
use voxcore::{VoxValuePool, VoxValuePoolKind, VoxValuePoolValueRef};
use voxj::VoxjValuePool;

/// Converts a [`VoxValuePool`] into a [`VoxjValuePool`], kind by kind, its
/// values in listing order. Every kind maps one to one; `json` values recurse
/// through [`voxj_value_from_vox_value`].
pub fn voxj_value_pool_from_vox_value_pool(value_pool: &VoxValuePool) -> VoxjValuePool {
    match value_pool.kind() {
        VoxValuePoolKind::Bool { .. } => VoxjValuePool::Bool(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Bool(flag) => flag,
                    other => unreachable!("a bool value pool yields bool values, not {other:?}"),
                })
                .collect(),
        ),

        VoxValuePoolKind::Float { .. } => VoxjValuePool::Float(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Float(number) => number,
                    other => unreachable!("a float value pool yields float values, not {other:?}"),
                })
                .collect(),
        ),

        VoxValuePoolKind::Int { .. } => VoxjValuePool::Int(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Int(number) => number,
                    other => unreachable!("an int value pool yields int values, not {other:?}"),
                })
                .collect(),
        ),

        VoxValuePoolKind::Json { .. } => VoxjValuePool::Json(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Json(value) => voxj_value_from_vox_value(value),
                    other => unreachable!("a json value pool yields json values, not {other:?}"),
                })
                .collect(),
        ),

        VoxValuePoolKind::String { .. } => VoxjValuePool::String(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::String(text) => text.to_owned(),
                    other => {
                        unreachable!("a string value pool yields string values, not {other:?}")
                    }
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec2Float { .. } => VoxjValuePool::Vec2Float(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec2Float(components) => *components,
                    other => unreachable!(
                        "a vec-2-float value pool yields vec-2-float values, not {other:?}"
                    ),
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec2Int { .. } => VoxjValuePool::Vec2Int(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec2Int(components) => *components,
                    other => unreachable!(
                        "a vec-2-int value pool yields vec-2-int values, not {other:?}"
                    ),
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec3Float { .. } => VoxjValuePool::Vec3Float(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec3Float(components) => *components,
                    other => unreachable!(
                        "a vec-3-float value pool yields vec-3-float values, not {other:?}"
                    ),
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec3Int { .. } => VoxjValuePool::Vec3Int(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec3Int(components) => *components,
                    other => unreachable!(
                        "a vec-3-int value pool yields vec-3-int values, not {other:?}"
                    ),
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec4Float { .. } => VoxjValuePool::Vec4Float(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec4Float(components) => *components,
                    other => unreachable!(
                        "a vec-4-float value pool yields vec-4-float values, not {other:?}"
                    ),
                })
                .collect(),
        ),

        VoxValuePoolKind::Vec4Int { .. } => VoxjValuePool::Vec4Int(
            value_pool
                .iter_values()
                .map(|(_, value)| match value {
                    VoxValuePoolValueRef::Vec4Int(components) => *components,
                    other => unreachable!(
                        "a vec-4-int value pool yields vec-4-int values, not {other:?}"
                    ),
                })
                .collect(),
        ),
    }
}
