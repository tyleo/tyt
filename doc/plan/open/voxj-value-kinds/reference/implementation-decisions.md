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

## Iteration 4 rulings

2026-08-02. The sentinel serde surface is one private `values` module
inside `voxj_value_pool.rs`, whole behind the `serde` cfg, with mirrored
`float` and `int` submodules: `serialize` and `deserialize` over the
scalar payload at each submodule's root, and an `array` submodule whose
pair is const-generic over the `[T; N]` payloads. The enum attaches
them per field: `#[serde(with = "values::float")]` on `Float`,
`#[serde(with = "values::float::array")]` on the float vectors, and the
int kinds likewise. The plumbing lives in the enum's own file because a
standalone with-module file cannot keep one public item per file: serde
fixes the module's two-function surface (owner ruling, 2026-08-02). The
enum variants sit in alphabetical order, matching the spec's kind
table (same ruling).

Two write-side rules the plan left implicit. The int write errors on a
value beyond `2^53 - 1` in magnitude, the parse's own cap, so a
`Vec<i64>` built in code cannot write an out-of-spec file. The float
integral-as-integer write admits values strictly below `i64::MAX as
f64`: that constant rounds up to `2^63`, and an inclusive bound would
saturate the cast and write `2^63` as `2^63 - 1`.

The text round-trip test pins three of the 8-bit linear-decode values
(the `k = 3`, `128`, and `252` codes) measured to mis-parse by one ULP
without `float_roundtrip` on serde_json 1.0.150.

## Iteration 5 rulings

2026-08-02. `check_value_pools` keeps no per-kind match at all. The only
surviving rule is non-emptiness, which `values_len` answers for every
kind, so the eleven-arm match the checklist anticipated has nothing to
do and does not appear.

`accepts_unbounded_and_hdr_value_pools` deletes alongside the eight
reject tests the survey counted. It accepted edge cases of the deleted
bound and color machinery, and on the new kinds it would only restate
the valid-file acceptance.

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

## Iteration 6 rulings

2026-08-02. voxcore keeps exactly the wire's value domains as construction
gates: a float value or vector component flaws when NaN, an int value or
component flaws beyond `2^53 - 1` in magnitude, and nothing else flaws. A
value the wire can never serialize is an error at its source, not at the
first save, and anything laxer would let a NaN live silently in the model.
The infinities are admitted where today's `Float` kind rejected them: the
wire spells them, so the model holds them. Ranges stay out entirely.

The constructors keep the kind-name spelling for the vector kinds:
`vec_3_float` for `vec-3-float`, digit separated, not the variant-derived
`vec3_float`.

## Iteration 7 rulings

2026-08-02. The one sRGB transfer lives in an internal helper pair:
`lin_srgba_f64_from_srgba_u8` decodes an 8-bit color to `TyLinSrgbaF64`
and `srgba_u8_from_lin_srgba_f64` encodes it back, gated on
`any(_color, _mesh)` because the color codecs and the mesh side both
cross the boundary. Every 8-bit crossing calls the pair: the goxl, mvox,
qb-family, and vmax importers, the voxelizer's fill colors, the texture
texel reads, the atlas texel writes, and `value_pool_color`. The u8
identity test and the independent transfer references from the old
write-side seam tests live beside the encode helper.

`MeshMaterial`'s two colors turn `TyLinSrgbaF64`, so the voxelizer's
material dedup and its color columns key on f64 bit patterns instead of
8-bit codes: per-texel sampling merges only bit-identical sampled
colors, the finer palette granularity the README accepts for storing
the sampled value exactly. Test fixtures that author colors as hex
decode them through the same helper, so byte-level expectations survive
unchanged.

2026-08-03, after the gate review. `OutOfRangeFactor` is
`OutOfRangeProperty`, with its file and the voxelizer's
`property_value`, `property_color`, and `property_value_pool` helpers:
the policy names voxj property values, not glTF wire fields, so the
factor spelling ends with the iteration 3 rename, the same ruling that
took `mesh_material.rs`'s `emissive_factor`. Prose citing the glTF
source material's own composition ("flat PBR factors") keeps its
spelling. vxl's `--out-of-range-factor` flag follows in iteration 8;
the checklist carries it. `MaterialChannel::Attribute` renames to
`Property` under the same ruling.

2026-08-03, after the gate review. The atlas bake reads colors linear:
`value_pool_lin_srgba_f64_color` is the base read, typed as the color it
reads, `value_pool_color` its 8-bit
encoding, and the emissive fold scales the exact stored color and
encodes once, so a texel rounds only at the boundary instead of before
and after the scale. The vmax writer keeps deriving `sic` from
`value_pool_color`'s quantized bytes on purpose: the vmax palette stores
the base color as u8 and Voxel Max glows in that stored color, so the
luminance ratio reads the color the file carries, not the exact value it
was quantized from.

2026-08-03, after the gate review. The atlas color path deals in the exact
color types end to end: `default_lin_srgba_f64_color` states the spec
default as the linear factor the glTF schema itself spells, the material
color resolvers return `TySrgbaU8`, and `component_byte` reads a typed
color's fields. Raw `[u8; 4]` survives only where the data is genuinely
bytes: the pixel buffer write and the packing arm, whose packed channels
are not an sRGB color, so a color type would mislabel them.

2026-08-03. `GltfRange::clamp` gates on `_mesh` beside its only callers,
the voxelizer's policy helpers, so a default-feature build carries no
dead clamp.

The vocabulary range check is `check_gltf_property_ranges`, a public
top-level function over a whole `VoxMain`. It walks every palette
property the vocabulary names and checks the values materials draw; an
undrawn value pool entry reaches no glTF factor and passes. It does not
absorb `scalar_range`: the table stays in `gltf_attributes.rs`,
upgraded to return `GltfRange` (an interval plus an admits-zero flag
spelling `ior`'s `{0} union [1, inf)`), and the check and the
voxelizer's `--out-of-range-factor` policy both read it, which retires
the old `1..` disagreement and admits `ior` `0` everywhere. The clamp
policy stays the voxelizer's: `factor_value_pool` keeps it for scalars
and the new `factor_color` extends it to the two color factors'
components, which the deleted u8 encode used to clamp silently. Both
glTF boundaries run the check itself: the export at the top of
`build_material_document` and the import at the end of `voxelize_mesh`,
the entry-side guarantee after the policy pass.
