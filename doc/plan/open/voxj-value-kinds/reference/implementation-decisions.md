# voxj value kinds implementation decisions

Code-level decisions made while executing the
[checklist](../checklist.md), recorded as they land. The plan-level
decisions and their rationale live in the [README](../README.md); this log
is for the finer implementation choices a reviewer of the Rust would want
explained. The checklist already names four that must land here:

1. The sentinel serde module's name and exact surface (the scalar and
   array forms, and how the adjacently tagged derive attaches them).
2. The vocabulary range check: its home, its signature, whether it
   absorbs `gltf_attributes.rs`'s `scalar_range`, and what becomes of the
   voxelizer's `--out-of-range-factor` clamp policy.
3. What value flaw checking voxcore keeps once the wire owns the domain
   rules.
4. Whether `internal/mesh/mesh_material.rs`'s `emissive_factor` field
   keeps its name.

## Iteration 1 rulings that diverge from the README

2026-08-02. The format defines no color, so the linear-light statement
is vocabulary convention and sits in the glTF conventions section
beside the ranges, not in Value Pool Kinds as a format-wide rule. The
emissive-composition paragraph folds into the conventions table. The
texture scope paragraph does not land: the absence of texture bindings
is by construction, and the boundary bake is tool documentation.

## Iteration 3 rulings

2026-08-02. `mesh_material.rs`'s `emissive_factor` field renames to
`emissive_color`. The factor-times-texture argument for keeping it proves
too much: every sibling field already carries the resolved-parameter name
(`base_color`, `transmission`, `occlusion`) while multiplying a texture
the same way, so the suffix marked one field out of eight for a property
all of them share. The glTF citations in the doc comments keep the wire
spellings.

Two comments in the renamed files stay on the glTF spellings because
they cite glTF's own composition, not the voxj vocabulary:
`from_vmax_file.rs`'s "`emissiveFactor` times `emissiveStrength` per
glTF" and `material_document.rs`'s doc on the KHR extension fields it
writes back. The test names
`per_texel_multiplies_the_base_color_factor_into_the_texel` and
`khr_extension_factors_round_trip_through_a_mesh_export` also keep
"factor": they describe the glTF material fields the import reads.

## Iteration 2 review rulings

2026-08-02, at the iteration 2 gate review. Three wording rulings from
the owner, folded into the spec commit. They override format-design.md,
which stopped being authoritative once the spec landed.

1. Between two phrasings carrying the same content, the shorter one
   stands. The Value Pools opening keeps the terse holds-values
   sentence, and the Value Pool Kinds intro drops the per-kind reading
   examples that restate the kind table.
2. Only the worst clause chains restructure into sentences: a run-on
   gluing three claims splits, and an idiomatic single colon or
   semicolon stays.
3. A self-explanatory table column gets no introducing sentence. The
   Range column note went.
4. The kinds speak the format's own vocabulary, value-shape, never JSON
   shape. JSON's types are coarser than the kinds: `int` and `float` are
   both one JSON number, and a `float` value's sentinels are strings.
   JSON stays only where the file's literal spelling is the subject.
