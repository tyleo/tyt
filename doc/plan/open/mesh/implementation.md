# Implementation

_Part of the [mesh plan](README.md): the code-facing notes. The design lives in
the other pages._

## The tracks

The work splits into four tracks, one per section below: vox-value-language,
voxsmith, vxl, and ty-preferences. Each track's numbered phases land in order,
each phase roughly a commit, and every phase leaves the workspace compiling with
its tests green.

Two seams keep the tracks parallel. The [record](#one-record) carries a whole
run from vxl into voxsmith, and its shape settles early, so voxsmith's tests
build records by hand and never wait on the flag or profile work.
vox-value-language's pipeline shape settles the same way, each stage landing
whole in its phase, so voxsmith waits on the language only from its evaluation
phase on.

The spine goes in early, across two tracks at once: vxl's new `mesh` builds a
minimal record, and voxsmith's entry point meshes it, geometry only, through
both containers. ty-preferences gates only vxl's profile and cascade phases,
because the `.vxlconfig` cascade and the embedded built-ins both read through
its jsonc deserializer. [The close](#the-close) waits on everything.

## One record

Flags and profiles meet in one record. Every option of
[`vxl mesh`](mesh.md#options) and every profile element lowers into the same
element of the record. An explicit flag replaces the profile element it collides
with. vxl only combines: it parses the flags, loads and expands the profiles,
resolves the `{file-stem}` templates, joins the value fragments into the
program, and hands the record over. voxsmith defines the record beside the entry
point that takes it. The call pairs the record with the loaded document state
and the selected object.

The record is plain data. Expressions ride as text until the entry point parses
them, and no element remembers the flag or the profile entry it came from:

```rust
use branded_id::{IdVec, U32Id};

/// The material index brand.
pub struct BMaterial;

/// The primitive index brand.
pub struct BPrimitive;

/// The target container.
pub enum Container {
    /// Binary glTF, `.glb`.
    Glb,

    /// glTF text, `.gltf`.
    Gltf,
}

/// The meshing strategy.
pub enum Method {
    /// One unmerged quad per boundary face.
    Culled,

    /// The fewest quads the run's values allow.
    Greedy,

    /// All six faces of every solid voxel.
    Naive,
}

/// The atlas canvas, counted in cells.
pub enum TextureShape {
    /// The near-square packing.
    Fit,

    /// A single row of cells.
    Line,

    /// The smallest square power of two.
    Pot,

    /// The smallest square.
    Square,

    /// An exact `n`x`n` canvas of cells.
    Exact(u32),
}

/// An array domain; the ladder runs bottom to top.
pub enum ArrayDomain {
    /// One entry per swatch.
    Swatch,

    /// One entry per solid voxel.
    Voxel,

    /// One entry per emitted face.
    Face,

    /// One entry per face corner.
    Corner,
}

/// The transfer a written value declares.
pub enum Transfer {
    /// Applies no transfer.
    Linear,

    /// Applies the sRGB transfer.
    Srgb,
}

/// A write's value.
pub struct WrittenValue {
    /// The expression the run evaluates.
    pub expression: String,

    /// The declared transfer.
    pub transfer: Transfer,
}

/// What computes into a binding.
pub enum Computation {
    /// Each entry's index in the domain.
    Index(ArrayDomain),

    /// Occlusion from the voxel geometry.
    Occlusion,

    /// Each voxel's grid position.
    VoxelPosition,
}

/// A binding the run computes into the environment.
pub struct ComputedBinding {
    /// The bound name.
    pub name: String,

    /// What computes into the name.
    pub computation: Computation,
}

/// A file's form; entries of one JSON file merge by path.
pub enum FileForm {
    /// A JSON entry under its name.
    Json { name: String },

    /// An 8-bit PNG.
    Png,
}

/// A loose file the run writes beside the mesh.
pub struct FileWrite {
    /// The file's relative name.
    pub file: String,

    /// The written value.
    pub value: WrittenValue,

    /// The file's form.
    pub form: FileForm,
}

/// A slot's source.
pub enum SlotSource {
    /// A file the run writes.
    File(String),

    /// An expression the run evaluates.
    Value(String),
}

/// A material slot write; the property resolves under the format's vocabulary.
pub struct SlotWrite {
    /// The destination property.
    pub property: String,

    /// What fills the property.
    pub source: SlotSource,
}

/// An extras entry's form.
pub enum ExtraForm {
    /// The entry references a texture.
    Image,

    /// The entry holds JSON.
    Json,
}

/// An extras entry's source.
pub enum ExtraSource {
    /// A referenced file.
    File(String),

    /// A written value.
    Value(WrittenValue),
}

/// An `extras.vxl.values` entry, on a material or the mesh.
pub struct ExtraWrite {
    /// The entry's name.
    pub name: String,

    /// The entry's form.
    pub form: ExtraForm,

    /// The entry's source.
    pub source: ExtraSource,
}

/// A vertex attribute write.
pub enum AttributeWrite {
    /// An attribute glTF defines; the vocabulary fixes its encoding.
    Builtin { attribute: String, expression: String },

    /// An underscore attribute.
    Custom { name: String, value: WrittenValue },
}

/// One material's elements.
pub struct MaterialRecord {
    /// The glTF `material.name`.
    pub name: Option<String>,

    /// The declared stream list; when absent, the list derives from use.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The slot writes.
    pub slots: Vec<SlotWrite>,

    /// The `extras.vxl.values` entries.
    pub extras: Vec<ExtraWrite>,
}

/// One primitive's elements.
pub struct PrimitiveRecord {
    /// The material the primitive draws with; `None` is no material.
    pub material_id: Option<U32Id<BMaterial>>,

    /// The select routing faces to the primitive.
    pub select: String,

    /// The `extras` `vxl.name`.
    pub name: Option<String>,

    /// Whether the primitive writes `NORMAL`.
    pub normal: bool,

    /// The declared stream list; when absent, the material's list applies.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The vertex attribute writes.
    pub attributes: Vec<AttributeWrite>,
}

/// A whole meshing run, with every flag and profile lowered in.
pub struct MeshRecord {
    /// The target container.
    pub container: Container,

    /// The meshing strategy.
    pub method: Method,

    /// The atlas canvas.
    pub texture_shape: TextureShape,

    /// One voxel's edge length in meters.
    pub voxel_size: f64,

    /// The computed bindings.
    pub computed_bindings: Vec<ComputedBinding>,

    /// The joined value program.
    pub program: String,

    /// The materials by index; the material count sets the length.
    pub materials: IdVec<BMaterial, MaterialRecord>,

    /// The primitives by index; the implicit primitive lowers as an entry
    /// whose `true` select takes every face, so the table holds at least one.
    pub primitives: IdVec<BPrimitive, PrimitiveRecord>,

    /// The loose files written beside the mesh.
    pub files: Vec<FileWrite>,

    /// The mesh's `extras.vxl.values` entries.
    pub mesh_extras: Vec<ExtraWrite>,
}
```

Attribution stays in vxl, which maps each record element to its origin as it
lowers. A voxsmith error names the element it rose from, and vxl rewraps the
error to name the flag or the profile entry.

The entry point bears the run. It parses, checks, and evaluates the program
through [vox-value-language](#vox-value-language), meshes the geometry, bakes
the atlases, and emits an output-independent document holding everything but the
container: the geometry streams, the primitives, the materials, the images, the
JSON payloads, and the loose files. The container writers serialize that
document, so `glb` and `gltf` differ only at this edge.

## vox-value-language

The language ships as vox-value-language, its own crate in utilities,
referencing none of the vxl crates. The crate owns the whole language as a
one-way pipeline over whole [programs](value-language.md#programs):

1. `parse` takes the program text to a syntax tree.
2. `check` takes the tree to a checked program answering every binding's type.
3. `eval` takes the checked program to an evaluated program answering every
   binding's value.

`eval` accepts only a checked program, so every type settles once in `check` and
evaluation reads the settled types instead of re-deriving them. The tree stays
internal; exporting it would split the semantics from the grammar, every new
function landing in two crates. A caller stops at the stage it needs:
[loading](profile-language.md#loading) stops after `parse`, and a hand-written
flag stops after `check` for its shape and dimension errors before anything
evaluates.

The names a program reads but never defines come in through an environment the
caller supplies, which is how the palette properties and the
[computed values](value-language.md#computed-values) load as bindings ahead of
the program:

```rust
let program = parse(&source)?;

let checked = check(program, &type_environment)?; // the types by name: shape, dimension, numeric type
let ao_type = checked.get("ao");

let evaluated = eval(&checked, &value_environment)?; // the values by name, plain or array
let ao_value = evaluated.get("ao");

let select = parse_expression("faceAvg(ao) < 0.7")?;
let checked_select = check_expression(&select, &checked)?; // the scope at the program's end
let select_value = eval_expression(&checked_select, &evaluated)?;
```

Each environment is plain data the crate defines, filled by voxsmith from the
effective palette and the computed values and written out by hand in the tests:

```rust
use branded_id::{IdVec, U32Id};
use std::collections::HashMap;

/// The swatch rung brand.
pub struct BSwatch;

/// The voxel rung brand.
pub struct BVoxel;

/// The face rung brand.
pub struct BFace;

/// What a value has one entry per; the ladder runs bottom to top.
pub enum Domain {
    /// A single entry.
    Plain,

    /// One entry per swatch.
    Swatch,

    /// One entry per solid voxel.
    Voxel,

    /// One entry per emitted face.
    Face,

    /// One entry per face corner.
    Corner,
}

/// The vec width.
pub enum Dimension {
    /// One component.
    Vec1,

    /// Two components.
    Vec2,

    /// Three components.
    Vec3,

    /// Four components.
    Vec4,
}

/// A component's type.
pub enum Scalar {
    /// A 32-bit float.
    F32,

    /// An unsigned 8-bit integer.
    U8,

    /// An unsigned 16-bit integer.
    U16,

    /// An unsigned 32-bit integer.
    U32,

    /// A boolean, vec1 only.
    Bool,

    /// A string, vec1 only.
    String,
}

/// A value's type.
pub struct Type {
    /// What the value has one entry per.
    pub domain: Domain,

    /// The vec width.
    pub dimension: Dimension,

    /// The component type.
    pub scalar: Scalar,
}

/// The types `check` reads.
pub struct TypeEnvironment {
    /// Each name's type.
    pub types: HashMap<String, Type>,
}

/// A value's entries, flattened component by component; a plain value holds
/// one entry.
pub enum Components {
    /// `f32` components.
    F32(Vec<f32>),

    /// `u8` components.
    U8(Vec<u8>),

    /// `u16` components.
    U16(Vec<u16>),

    /// `u32` components.
    U32(Vec<u32>),

    /// `bool` components.
    Bool(Vec<bool>),

    /// `String` components.
    String(Vec<String>),
}

/// A value. The constructor errors on a component count that disagrees with
/// the domain and dimension and on a bool or string above vec1, so the fields
/// stay private.
pub struct Value {
    /// What the value has one entry per.
    domain: Domain,

    /// The vec width.
    dimension: Dimension,

    /// The entries, flattened.
    components: Components,
}

/// The tables the reductions and climbs walk between the domains; the corner
/// rung needs none because every face owns four corners, in face order.
pub struct Groupings {
    /// Each voxel entry's swatch entry.
    pub voxel_swatches: IdVec<BVoxel, U32Id<BSwatch>>,

    /// Each face entry's voxel pieces, several where a merged face spans
    /// voxels.
    pub face_voxels: IdVec<BFace, Vec<U32Id<BVoxel>>>,
}

/// The values `eval` reads.
pub struct ValueEnvironment {
    /// Each name's value.
    pub values: HashMap<String, Value>,

    /// The tables the reductions and climbs walk.
    pub groupings: Groupings,
}
```

The crate performs no side effects; parsing, checking, and evaluating stay in
memory. It carries no `Dependencies` trait, no `impl` feature, and no default
implementation, because an injection surface would claim an I/O boundary the
crate does not have.

vxl assembles the program: a `;` appended to every `--value` and profile values
fragment, the fragments joined in flag order with a `--profile`'s values first,
each origin kept, so a parse error names the flag or the profile entry rather
than a position in the joined text. A writer's `<src-expr>` and a `--primitive`
select ride the [record](#one-record) as expression text and go through the
expression siblings, checking and evaluating against the checked and the
evaluated program, so an expression reads the scope at the program's end beside
every name the environment supplies.

To the crate an array is a length, so the palette is voxsmith's interpretation:
voxsmith binds the effective palette into the environment in atlas-texel order
and keeps the edges, the transfer encoding, the png sizing, and the slot
cross-checks, which is where the linear-floats rule already puts them. The
voxel, face, and corner [domains](value-language.md#domains) are more lengths to
evaluate over, voxsmith supplying the groupings the reductions read, so the
crate never learns what a domain means.

### 1. The lexer

The crate lands with the token rules: maximal munch over the numbers and
operators, the attached postfixes, the backtick-quoted names, and the string
literals; see the [notes](value-language.md#notes).

### 2. The parser

`parse` lands over the [grammar](value-language.md#grammar)'s compact form, a
program of `;`-terminated bindings down through the expression rules, and
`parse_expression` starts where the expression rules do.

### 3. The checker

`check` lands over the grammar's checking rules, answering every binding's
shape, dimension, and numeric type over the names the environment supplies, and
`check_expression` runs the same rules in a checked program's end scope.

### 4. The evaluator

`eval` lands last, computing the operators, the functions, the reductions, and
the climbs over the environment's values, and `eval_expression` computes in an
evaluated program's end scope.

## voxsmith

### 1. The entry point

The entry point takes the [record](#one-record), meshes geometry alone through
the kept core, and hands the document to the two container writers. This is
voxsmith's half of the spine: the record, the document, and the writers exist
from here on, and the later phases thicken them.

### 2. The environment

The effective palette binds its properties in atlas-texel order, and the
[computed values](value-language.md#computed-values) bind on request.

### 3. The evaluation

The entry point supplies the environment and the groupings, and `check` and
`eval` run the program over the [domains](value-language.md#domains).

### 4. The geometry

The kept greedy core learns the [merge rules](mesh.md#atlases) the run's values
set.

### 5. The atlases and streams

The four layouts shape onto the canvas, the UVs land at texel centers, and every
`texCoord` [derives](mesh.md#uv-streams).

### 6. The primitives

The selects [partition the faces](mesh.md#primitives-and-materials), and each
primitive carries its own attributes and streams.

### 7. The files

The png encoder grows grey, grey-alpha, RGB, and the transfer chunks, and the
file writers land loose pngs and JSON beside the mesh.

### 8. The slots

The slot writers fill
[material fields and textures](value-language.md#material-slots), embedding a
value or referencing a written file under the format's vocabulary and its fixed
encodings.

### 9. The extras

The extras writers land named entries under `extras.vxl.values` in their four
forms, on the materials and on the [mesh](mesh.md#palettes).

### 10. The vertex attributes

The primitive writers land
[`COLOR_0` and the underscore attributes](value-language.md#vertex-attributes)
on the corners, with lower domains climbing in.

### 11. The containers

The document serializes to either target. `glb` packs images and geometry into
the binary chunk, `gltf` writes data URIs, and both hand back the run's loose
files to land beside the mesh.

## vxl

### 1. `mesh-old`

The shipped `mesh` renames whole, the subcommand, its module, and its flag
types, and the shipped voxsmith material path keeps serving it untouched, so a
working bake stands beside the rewrite until [the close](#the-close) removes
both together.

### 2. The spine

A new `mesh` lands beside it, taking the input, the output, `--to`, `--from`,
and the selectors, building a minimal [record](#one-record), and calling the
[entry point](#1-the-entry-point). The command runs end to end from here on,
geometry only.

### 3. The flags

The full surface of [`vxl mesh`](mesh.md#options) lands in clap, each flag
lowering into its element of the record. vxl checks what flags alone decide, and
the entry point errors on an element it cannot mesh yet, naming it, so the whole
surface lands ahead of the meshing work.

### 4. The profiles

The [schema](profile-language.md#schema) types land with serde, the
[built-ins](profile-language.md#built-in-profiles) embed, and `--profile` and
`--values-from` expand into record elements under the
[flag-beats-profile rule](profile-language.md#expansion). The checks fire at the
[stages](profile-language.md#loading) the profile language sets.

### 5. The cascade

The `.vxlconfig` layers join through [ty-preferences](#ty-preferences), and the
track closes with profiles loading through the whole cascade.

## ty-preferences

[Loading](profile-language.md#loading) reads `.vxlconfig` through the
preferences crate, which asks five things of it, one phase each in landing
order. The tyt call sites track each change as it lands.

### 1. The jsonc read

A `jsonc` feature adds a read that strips comments with json_comments ahead of
serde_json and carries the json_comments dependency; json_comments handles
comments alone, which is why a trailing comma stays an error. The `.tytconfig`
loaders keep the plain json read; vxl enables the feature for everything it
loads, and the phase lands first because the built-in profiles parse through its
read.

### 2. The file name

The loaders currently hardcode `.tytconfig`, so the file name becomes a
parameter and every tool names its own config file: vxl passes `.vxlconfig` and
the `mesh` key its envelope already defines.

### 3. The working-directory layer

The cascade currently ends at the git root, so the working directory becomes a
third layer. The `.vxlconfig` files then read as a cascade: the home
`~/.vxlconfig`, then the repository's `<git-root>/.vxlconfig`, then the working
directory's `.vxlconfig`. Outside a git repository the repository layer is
absent. [Loading](profile-language.md#loading) sets how profiles read through
the layers.

### 4. The implementation

The `impl` feature currently pulls in tyt-injection, which carries terminal,
image, and network crates, and loading a config needs only serde_json and an
atomic file write, so the crate carries its own optional implementation and the
existing tyt tool passes an implementation in.

### 5. The move

tyt-preferences becomes ty-preferences and moves into utilities, joining the
family vxl already lives in, so vxl depends on it without touching the tyt
crates. The move lands last, so the crate arrives in utilities already
independent.

## The close

`mesh-old` deletes whole, the command, its flag types, and the voxsmith material
path only it still calls, taking the first two [code deletions](#code-deletions)
with it; the third lands inside voxsmith's image writer. A sweep follows the
delete: every `.rs` file the shipped implementation reached is checked for
remaining callers, and the files nothing reaches anymore go too. The plan closes
when the [worked examples](examples.md) run as written.

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
