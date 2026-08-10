# Implementation

_Part of the [mesh plan](README.md): the code-facing notes. The design
lives in the other pages._

## The language crate

The language ships as its own crate, referencing none of the vxl
crates. The crate owns the whole language: `parse` takes text to a
syntax tree, `check` takes the tree and each name's type to the
result type, and `eval` takes the tree and each name's value to the
result. The name information comes in through an environment the
caller supplies:

```rust
let tree = parse("rgb(occlusionStrength, roughnessFactor, metallicFactor)")?;

let ty = check(&tree, &env)?; // env: name -> Option<Type>, shape x dimension

let value = eval(&tree, &env)?; // env: name -> Option<Value>, plain or array
```

To the crate an array is a length, so the palette is vxl's
interpretation: vxl binds the effective palette into the environment in
atlas-texel order and keeps the edges, the transfer encoding, the png
sizing, and the slot cross-checks, which is where the linear-floats
rule already puts them. The face and corner
[domains](value-language.md#domains) are more lengths to evaluate
over, vxl supplying the four-per-face grouping the face reductions
read, so the crate never learns what a domain means.

The tree stays internal; `parse`, `check`, and `eval` are the API.
Exporting the tree would split the semantics from the grammar, every
new function landing in two crates. `check` stays public beside `eval`:
[loading](profile-language.md#loading) wants `parse` alone, and a
hand-written flag wants its shape and dimension errors before anything
evaluates.

## ty-preferences

[Loading](profile-language.md#loading) reads `.vxlconfig` through the
preferences crate, which asks four things of it. tyt-preferences
becomes ty-preferences and moves into utilities, joining the family vxl
already lives in, so vxl depends on it without touching the tyt crates.
Its `impl` feature currently pulls in tyt-injection, which carries
terminal, image, and network crates, and loading a config needs only
serde_json and an atomic file write, so the crate carries its own
optional implementation and the existing tyt tool passes an
implementation in. Its loaders currently hardcode `.tytconfig`, so the
file name becomes a parameter and every tool names its own config file,
vxl passing `.vxlconfig` and the `mesh` key its envelope already
spells. And its reads strip comments with json_comments ahead of
serde_json, so every config it loads is jsonc, `.tytconfig` included;
json_comments handles comments alone, which is why a trailing comma
stays an error.

## Retired flags

The design retires the shipped map surface. `--texture` gives way to
the profile flags, `--value-profile` and `--output-profile`, a split
that also retires the interim `--profile` and `--write-profile` pair:
a values mixin is a value profile, and the writes ride the single
output profile. The slots and writers replace `--texture-storage`: an
image goes where its flag puts it, and the old `both` is a
`--write-file-png-value` beside a `--write-material-slot-value`.
`--value` replaces `--texture-map`, and backtick quoting reaches a
voxel-json key directly, retiring `--define-property`. The two naming flags
retire as well: a profile names its own files through `{file-stem}`
templates, an exact rename is a `.vxlconfig` profile respelling the
template, and `--file-stem` replaces the prefix flag; a hand-written
writer names its own file inline. Among the older designs,
`--write-primitive-builtin-value` and
`--write-primitive-custom-value` supersede the `--vertex`,
`--vertex-target`, and `--vertex-map` carriers, and expressions
supersede the three `--computed-occlusion-*` tuning flags.
`--write-mesh-extra-json-value` supersedes the `palette-index`
carrier and retires `palette-layers` outright: layers end at the
flatten, and a runtime grouping is an authored int property written
as rows. `--palette-storage` retires into flag combinations:
`embedded` is `--write-mesh-extra-json-value`, `external` is
`--write-file-json-value` beside a `--write-mesh-extra-json-file`
pointer, and `both` is the two side by side.

## Code deletions

1. voxsmith's `MaterialSlot::OcclusionMetallicRoughness`: two
   `--write-material-slot-value` flags naming one value share the
   image, so the combined variant goes.
2. The `extras.vxl.maps` emission: nothing slotless embeds anymore,
   so the automatic listing has no producer;
   `--write-material-extra-image-value` is its deliberate
   replacement.
3. `png_bytes.rs` hardcodes RGBA; the sized-to-value rule needs grey,
   grey-alpha, and RGB, and the transfer chunks want the png crate's
   `set_source_srgb` and `set_source_gamma`.
