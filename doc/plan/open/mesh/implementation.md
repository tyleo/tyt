# Implementation

_Part of the [mesh plan](README.md): the code-facing notes. The design lives in
the other pages._

## The tracks

The work splits into four tracks, one per section below: vox-value-language,
voxsmith, vxl, and ty-preferences. Each track's subsections are its steps, in
landing order, and every landing leaves the workspace compiling with its tests
green.

Two seams keep the tracks parallel. The [record](#one-record) carries a whole
run from vxl into voxsmith, and its shape settles early, so voxsmith's tests
build records by hand and never wait on the flag or profile work.
vox-value-language's API settles the same way, `parse`, `check`, and `eval` over
environments the caller hands in as callbacks, so voxsmith codes against the
signatures while the implementation fills in behind them.

The spine goes in early, across two tracks at once: vxl's new `mesh` builds a
minimal record, and voxsmith's entry point meshes it, geometry only, through
both containers. ty-preferences gates only vxl's profile step, because the
`.vxlconfig` cascade and the embedded built-ins both read through its jsonc
deserializer. [The close](#the-close) waits on everything.

## One record

Flags and profiles meet in one record. Every option of
[`vxl mesh`](mesh.md#options) and every profile element lowers into the same
element of the record, an explicit flag replacing the element it collides with,
so vxl only combines: it parses the flags, loads and expands the profiles,
resolves the `{file-stem}` templates, gathers the value fragments with their
origins, and hands the record over. voxsmith defines the record beside the entry
point that takes it, and the call pairs the record with the loaded document
state and the selected object.

The entry point bears the run. It parses, checks, and evaluates the program
through [vox-value-language](#vox-value-language), meshes the geometry, bakes
the atlases, and emits an output-independent document holding everything but the
container: the geometry streams, the primitives, the materials, the images, the
JSON payloads, and the loose files. The container writers serialize that
document, so `glb` and `gltf` differ only at this edge. An error deep in the run
still names the flag or the profile entry it rose from because the record keeps
every fragment's origin.

## vox-value-language

### The API

The language ships as vox-value-language, its own crate in utilities,
referencing none of the vxl crates. The crate owns the whole language: `parse`
takes text to a syntax tree, a [program](value-language.md#programs) of bindings
or a bare expression each through its own entry point, `check` takes the tree
and each name's type to the result type, and `eval` takes the tree and each
name's value to the result. The name information comes in through an environment
the caller supplies:

```rust
let tree = parse("rgb(occlusionStrength, roughnessFactor, metallicFactor)")?;

let ty = check(&tree, &env)?; // env: name -> Option<Type>, shape, dimension, numeric type

let value = eval(&tree, &env)?; // env: name -> Option<Value>, plain or array
```

The tree stays internal; `parse`, `check`, and `eval` are the API. Exporting the
tree would split the semantics from the grammar, every new function landing in
two crates. `check` stays public beside `eval`:
[loading](profile-language.md#loading) wants `parse` alone, and a hand-written
flag wants its shape and dimension errors before anything evaluates.

The API settles first and the implementation follows: the lexer, the parser, the
checker, and the evaluator fill in behind it, implementing the
[grammar](value-language.md#grammar)'s compact form and its checking rules.

### The callers

vxl assembles the program: a `;` appended to every `--value` and profile values
fragment, the fragments joined in flag order with a `--profile`'s values first,
each origin kept, so a parse error names the flag or the profile entry rather
than a position in the joined text.

To the crate an array is a length, so the palette is voxsmith's interpretation:
voxsmith binds the effective palette into the environment in atlas-texel order
and keeps the edges, the transfer encoding, the png sizing, and the slot
cross-checks, which is where the linear-floats rule already puts them. The
voxel, face, and corner [domains](value-language.md#domains) are more lengths to
evaluate over, voxsmith supplying the groupings the reductions read, so the
crate never learns what a domain means.

## voxsmith

### The entry point

The entry point takes the [record](#one-record), meshes geometry alone through
the kept core, and hands the document to the two container writers. This is
voxsmith's half of the spine: the record, the document, and the writers exist
from here on, and the later steps thicken them.

### The meshing

The run fills in phase by phase.

1. The environment: the effective palette binds its properties in atlas-texel
   order, and the [computed values](value-language.md#computed-values) bind on
   request.
2. Evaluation: the entry point supplies the environment and the groupings, and
   `check` and `eval` run the program over the
   [domains](value-language.md#domains).
3. Value-driven geometry: the kept greedy core learns the
   [merge rules](mesh.md#atlases) the run's values set.
4. The atlases and streams: the four layouts shape onto the canvas, the UVs land
   at texel centers, and every `texCoord` [derives](mesh.md#uv-streams).
5. The primitives: the selects
   [partition the faces](mesh.md#primitives-and-materials), and each primitive
   carries its own attributes and streams.
6. The writers: values land in images, material slots, extras entries, JSON
   files, and vertex attributes, and the png encoder grows grey, grey-alpha,
   RGB, and the transfer chunks.

### The containers

The document serializes to either target. `glb` packs images and geometry into
the binary chunk, `gltf` writes data URIs, and both hand back the run's loose
files to land beside the mesh.

## vxl

### `mesh-old`

The shipped `mesh` renames whole, the subcommand, its module, and its flag
types, and the shipped voxsmith material path keeps serving it untouched, so a
working bake stands beside the rewrite until [the close](#the-close) removes
both together.

### The spine

A new `mesh` lands beside it, taking the input, the output, `--to`, `--from`,
and the selectors, building a minimal [record](#one-record), and calling the
[entry point](#the-entry-point). The command runs end to end from here on,
geometry only.

### The flags

The full surface of [`vxl mesh`](mesh.md#options) lands in clap, each flag
lowering into its element of the record. vxl checks what flags alone decide, and
the entry point errors on an element it cannot mesh yet, naming it, so the whole
surface lands ahead of the meshing work.

### The profiles

The [schema](profile-language.md#schema) types land with serde, the
[built-ins](profile-language.md#built-in-profiles) embed, and `--profile` and
`--values-from` expand into record elements under the
[flag-beats-profile rule](profile-language.md#expansion). The checks fire at the
[stages](profile-language.md#loading) the profile language sets, and the
`.vxlconfig` cascade closes the track.

## ty-preferences

[Loading](profile-language.md#loading) reads `.vxlconfig` through the
preferences crate, which asks five things of it, one subsection each in landing
order. The tyt call sites track each change as it lands.

### The jsonc read

The crate's reads strip comments with json_comments ahead of serde_json, so
every config it loads is jsonc, `.tytconfig` included; json_comments handles
comments alone, which is why a trailing comma stays an error. The read lands
first because vxl's built-in profiles parse through it.

### The file name

The loaders currently hardcode `.tytconfig`, so the file name becomes a
parameter and every tool names its own config file: vxl passes `.vxlconfig` and
the `mesh` key its envelope already defines.

### The working-directory layer

The cascade currently ends at the git root, so the working directory becomes a
third layer. The `.vxlconfig` files then read as a cascade: the home
`~/.vxlconfig`, then the repository's `<git-root>/.vxlconfig`, then the working
directory's `.vxlconfig`. Outside a git repository the repository layer is
absent. [Loading](profile-language.md#loading) sets how profiles read through
the layers.

### The implementation

The `impl` feature currently pulls in tyt-injection, which carries terminal,
image, and network crates, and loading a config needs only serde_json and an
atomic file write, so the crate carries its own optional implementation and the
existing tyt tool passes an implementation in.

### The move

tyt-preferences becomes ty-preferences and moves into utilities, joining the
family vxl already lives in, so vxl depends on it without touching the tyt
crates. The move lands last, so the crate arrives in utilities already
independent.

## The close

`mesh-old` deletes whole, the command, its flag types, and the voxsmith material
path only it still calls, taking the first two [code deletions](#code-deletions)
with it; the third lands inside voxsmith's image writer. The plan closes when
the [worked examples](examples.md) run as written.

## Retired flags

The design retires the shipped map surface. `--texture` gives way to the profile
flags, `--values-from` and `--profile`, which replace the interim `--profile`
and `--write-profile` pair: a values mixin is any profile's values through
`--values-from`, and the writes ride the one `--profile`. The slots and writers
replace `--texture-storage`: an image goes where its flag puts it, and the old
`both` is a `--write-file-png-value` beside a `--write-material-slot-value`.
`--value` replaces `--texture-map`, and backtick quoting reaches a voxel-json
key directly, retiring `--define-property`. The two naming flags retire as well:
a profile names its own files through `{file-stem}` templates, an exact rename
is a `.vxlconfig` profile redefining the template, and `--file-stem` replaces
the prefix flag; a hand-written writer names its own file inline. Among the
older designs, `--write-primitive-builtin-value` and
`--write-primitive-custom-value` supersede the `--vertex`, `--vertex-target`,
and `--vertex-map` carriers, and expressions supersede the three
`--computed-occlusion-*` tuning flags. `--write-mesh-extra-json-value`
supersedes the `palette-index` carrier and retires `palette-layers` outright:
layers end at the flatten, and a runtime grouping is an authored int property
written as rows. `--palette-storage` retires into flag combinations: `embedded`
is `--write-mesh-extra-json-value`, `external` is `--write-file-json-value`
beside a `--write-mesh-extra-json-file` pointer, and `both` is the two side by
side. `--atlas` retires into the `--material-uv` contract: a material's stream
list chooses its textures' layouts, `unwrap` becomes a `face`-only list per
material, and the palette bake is the derived default.

## Code deletions

1. voxsmith's `MaterialSlot::OcclusionMetallicRoughness`: two
   `--write-material-slot-value` flags naming one value share the image, so the
   combined variant goes.
2. The `extras.vxl.maps` emission: nothing slotless embeds anymore, so the
   automatic listing has no producer; `--write-material-extra-image-value` is
   its deliberate replacement.
3. `png_bytes.rs` hardcodes RGBA; the sized-to-value rule needs grey,
   grey-alpha, and RGB, and the transfer chunks want the png crate's
   `set_source_srgb` and `set_source_gamma`.
