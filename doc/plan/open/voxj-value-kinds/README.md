# voxj value kinds

Status: **design settled, unplanned.** One rule for the voxel-json
[value pool kinds](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#value-pool-kinds),
replacing the arity spelling and the two linear color kinds. This page is the
design; the ordered steps are not written yet.

The kind vocabulary spells three things into eleven names: what a value is,
how many components it has, and whether it needs decoding. Two of those do not
belong in a name. Arity is a count, so `srgb-float` and `srgba-float` are one
kind twice, and a 2-component value has no spelling at all short of inventing
`sr` and `srg`. And `linear-rgb-float` names a transfer but no primaries,
which identifies no color. Between them the format has no home for non-color
data at all: a normal's components run `-1..1`, which every color kind
forbids.

## The rule

A kind names what a consumer must do with the value, and a kind whose value is
an array declares how long that array is.

1. **A scalar kind carries no count.** `json`, `string`, `bool`, `int`, and
   `float` are single values. A scalar is not a one-element array: `0.5` and
   `[0.5]` are different JSON, so `components: 1` would be a shape switch
   rather than a count.
2. **An array kind declares `components`,** an integer `2..4` for a vector and
   `3..4` for a color, required, with no default.
3. **A kind says whether the value needs decoding, not what color space it
   lives in.** Every color in voxj uses sRGB primaries and the D65 white
   point; the format states that once rather than repeating it in each name.
   What is left for a name to carry is whether the sRGB transfer is applied.

Rule 3 is what removes the linear color kinds. A linear value needs no
decoding, so it is used as-is, which is exactly what a plain vector is. A
linear color and a normal are the same thing to every consumer, and the only
kinds left that a consumer treats specially are the two that must be decoded.

## The vocabulary

| Kind           | Components | Value                                      |
| -------------- | ---------- | ------------------------------------------ |
| `json`         | scalar     | any JSON, including null                   |
| `string`       | scalar     | string                                     |
| `bool`         | scalar     | boolean                                    |
| `int`          | scalar     | integer-valued number, `min`/`max`         |
| `float`        | scalar     | finite number, `min`/`max`                 |
| `vector-float` | 2-4        | array of finite numbers, `min`/`max`       |
| `srgb-float`   | 3-4        | array of sRGB components in `0..1`, decode |
| `srgb-hex`     | 3-4        | `#RRGGBB` or `#RRGGBBAA`, decode           |

Eight kinds, down from eleven. `srgba-float`, `srgba-hex`,
`linear-rgb-float`, and `linear-rgba-float` are gone: the first two fold into
their `components: 4` form, and the last two become `vector-float`.

```jsonc
// before
{ "kind": "srgba-hex", "values": ["#FF0000FF"] }
{ "kind": "linear-rgb-float", "values": [[2, 0, 0]] }

// after
{ "kind": "srgb-hex", "components": 4, "values": ["#FF0000FF"] }
{ "kind": "vector-float", "components": 3, "min": 0, "max": "none",
  "values": [[2, 0, 0]] }

// newly spellable: non-color data, which no color kind could hold
{ "kind": "vector-float", "components": 3, "min": -1, "max": 1,
  "values": [[0, 0, 1], [1, 0, 0]] }
```

The last one is the point of `min`/`max` on a vector. A normal's components
run `-1..1`, which `linear-rgb-float` forbade with its `>= 0` rule, so the
format could not hold one. Non-color data is what glTF calls Non-Color Data
and Blender calls Non-Color, and this repo already settled that such channels
are numbers rather than colors in the
[ty-color-model plan](../../closed/ty-color-model/README.md).

Inferring `components` from the values was considered and rejected. It is
always possible, since `values` is non-empty, but it makes a pool's shape a
property of its data rather than a declaration, and voxj's rule is that
declaring a kind is what enables validation.

## What still reads a color

Dropping the linear color kinds means a consumer can no longer ask a pool
whether it holds a color. It asks the property instead: the
[glTF vocabulary](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#gltf-conventions)
says `baseColorFactor` and `emissiveFactor` are colors, and the kind says only
whether to decode first. Two consequences the implementation must carry:

1. `pool_color` accepts `vector-float` at 3 or 4 components as a color already
   in linear light, alongside the two sRGB kinds it decodes. Without that the
   mesh atlas bake would read a linear `baseColorFactor` as no color at all
   and fall back to white.
2. `palette show` loses swatch inference for a *custom* linear color, since a
   `vector-float` 3 is a normal or a color and the pool no longer says which.
   It keeps swatches for the built-ins by property name, and `--type` already
   exists on `show` to assert the type of a custom key.

## The Rust shape

The wire carries `components`; the Rust types must not. A stored count beside
a typed column is a second source of truth that can disagree with the column's
width, so the arity is the variant. Nesting the arity keeps the kind list
short:

```rust
pub enum VoxValuePoolKind {
    Float { min: VoxBound, max: VoxBound, values: IdField<BVoxValuePoolValue, f64> },
    Vector { min: VoxBound, max: VoxBound, values: VoxVectorColumn },
    Srgb { values: VoxColorColumn },
    // ...
}

pub enum VoxVectorColumn {
    C2(IdField<BVoxValuePoolValue, [f64; 2]>),
    C3(IdField<BVoxValuePoolValue, [f64; 3]>),
    C4(IdField<BVoxValuePoolValue, [f64; 4]>),
}
```

The flat alternative, one variant per (kind, arity) pair, reads more plainly
at the definition and costs a much longer match at every use.

Two facts worth not rediscovering. voxcore already collapses hex into
float: its kind has `Srgb` / `Srgba` / `LinearRgb` / `LinearRgba` and no hex
variant, because `from_voxj_file` canonicalizes an `srgba-hex` pool on read
and vxl's `--color-format` chooses the spelling on write. And the arity split
already exists in memory, so `Srgb` / `Srgba` merging into one variant over a
column is a narrowing, not a new axis. The read-side `VoxValuePoolValueRef`
needs the same treatment.

## Compatibility

Breaking. voxj rejects an unrecognized `kind` and an unknown pool key by rule,
so every document with an `srgba-*` or `linear-*` pool, or without
`components`, fails to load. The repo has no external consumers and the voxj
redesign already renamed the property vocabulary in place at `version: 1`, so
the recommendation is the same hard break with no aliases, regenerating the
fixtures.

## Blast radius

A starting list from one pass of greps, to be confirmed at the keyboard:

1. `projects/voxel-codecs/voxj/docs/voxel-json-file-format.md`: the kind table,
   the notes, validation rules 9 and 10, the TypeScript schema, the examples,
   and a new sentence fixing the primaries format-wide.
2. `projects/voxel-codecs/voxj/src/voxj_value_pool.rs`: the wire enum's six
   color variants and their serde.
3. `projects/voxel-codecs/voxj-codec/src/`: `validate_voxj_file.rs` and
   `check_voxj_file.rs`, the per-kind value checks plus a `components` rule.
4. `projects/utilities/voxcore/src/`: `vox_value_pool_kind.rs`,
   `vox_value_pool.rs`, `vox_main.rs`.
5. `projects/utilities/voxsmith/src/`: `internal/pool_color.rs` (see
   [what still reads a color](#what-still-reads-a-color)),
   `convert/voxj/from_voxj_file.rs`, `convert/voxj/color_format.rs`,
   `internal/voxj/voxj_value_pool_from_vox_value_pool.rs`, the atlas bake's
   color reads, and the palette reduction's.
6. `projects/utilities/vxl/src/`: `utilities/voxj_color_format.rs`,
   `implementation/palette_show.rs` (the swatch classification), and
   `implementation/mesh_object.rs` (its channel-kind classification).
7. Test fixtures and any checked-in `.voxj` / `.voxjz` assets.

## Open decisions

To settle with the owner before implementing:

1. Whether `vector-int` is needed, or `vector-float` covers integer vectors.
2. Whether `--color-format` keeps its `linear` value, which now names a kind
   that no longer exists.
3. `version` stays `1` with a hard break, or bumps to `2`.
4. The Rust modeling: nested column enums or flat per-shape variants.
5. Whether the in-memory model keeps collapsing hex into float, which it does
   today, or starts carrying the spelling.
