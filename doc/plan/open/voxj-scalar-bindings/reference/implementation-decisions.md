# voxj scalar bindings implementation decisions

Code-level decisions made while executing the
[checklist](../checklist.md), recorded as they land. The plan-level decisions
and their rationale live in the [README](../README.md#decisions); this log is
for the finer implementation choices a reviewer of the Rust would want
explained, for example the final Rust binding type names, how voxcore stores
scalar bindings, and whether the glTF converter emits them.

## voxcore `VoxPalette` target surface

Owner, 2026-07-15, ahead of phase 5. The binding namespace split means
`VoxPalette` (`projects/utilities/voxcore/src/vox_palette.rs`) grows from
today's `binding_ids` / `bindings` / `material_ids` / `materials` /
`by_attribute` to:

1. `array_binding_ids`
2. `scalar_binding_ids`
3. `material_ids`
4. `materials`
5. `array_binding_by_property`
6. `scalar_binding_by_property`
7. `binding_by_property`, a `HashMap` from property `String` to an
   array-or-scalar-tagged binding id

Materials stay column-major over the array-binding ids only. The map names
follow the format-wide attribute-to-property rename (README decision 11,
2026-07-16); the owner originally specified them as `*_by_attribute`.

The 2026-07-17 wire rename (`arrayProperties` / `scalarProperties`, `name` /
`valuePool` / `valueIndex`; README decision 12) postdates this surface;
whether the Rust names follow (`array_property_ids`, `property_by_name`, and
so on) settles in phase 5.
