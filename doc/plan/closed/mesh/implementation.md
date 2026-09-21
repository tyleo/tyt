# Implementation

_Part of the [mesh plan](README.md): the code-facing notes. The design lives in
the other pages._

## The tracks

The open work splits into three tracks: vox-value-language, voxsmith, and vxl.
Each track's numbered phases land in order, each phase roughly a commit, and
every phase leaves the workspace compiling with its tests green. The mesh
crates the tracks build on and the `mesh` tail that already runs through them
are [closed](#closed).

Three seams keep the tracks parallel. The [record](#one-record) carries a whole
run from vxl into voxsmith, and its shape settles early, so voxsmith's tests
build records by hand and never wait on the flag or profile work. The
[document](#meshdoc) carries meshes between voxsmith and meshconv, and it is
already in place, so both sides build against it from the start.
vox-value-language's pipeline shape settles the same way, each stage landing
whole in its phase, so voxsmith waits on the language only from its evaluation
phase on.

The spine goes in early, across two tracks at once: vxl's new `mesh` builds a
minimal record and voxsmith meshes it into a geometry-only document, which vxl
saves through meshconv as the shipped `mesh` already does. Profiles load
through [ty-preferences](#ty-preferences) as it stands, so no phase waits on
it. [The close](#the-close) waits on everything.

## One record

Flags and profiles meet in one record. Every profile element and every option of
[`vxl mesh`](mesh.md#options) but `--to` and `--from` lowers into the same
element of the record. An explicit flag replaces the profile element it collides
with. `--to` and `--from` stay in vxl, which reads and writes the files, so the
record carries no file format. vxl only combines: it parses the flags, loads and
expands the profiles, resolves the `{file-stem}` templates, joins the value
fragments into the program, and hands the record over. voxsmith defines the
record beside the entry point that takes it. The call pairs the record with the
loaded state and the selected object.

The record is plain data. Expressions ride as text until the entry point parses
them, and no element remembers the flag or the profile entry it came from:

```rust
use branded_id::{IdVec, U32Id};
use meshdoc::{BMeshMaterial, BMeshPrimitive};

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

/// A file the run lands in the document, written beside the mesh.
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

/// A material slot write; the property names a modeled material field.
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

/// A named property of a material or the object, which the glTF bridge
/// writes under `extras.vxl.values`.
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
    /// A stream the document models, which fixes its encoding.
    Builtin { attribute: String, expression: String },

    /// An underscore attribute.
    Custom { name: String, value: WrittenValue },
}

/// One material's elements.
pub struct MaterialRecord {
    /// The material's name.
    pub name: Option<String>,

    /// The declared stream list; when absent, the list derives from use.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The slot writes.
    pub slots: Vec<SlotWrite>,

    /// The named properties.
    pub extras: Vec<ExtraWrite>,
}

/// One primitive's elements.
pub struct PrimitiveRecord {
    /// The material the primitive draws with; `None` is no material.
    pub material_id: Option<U32Id<BMeshMaterial>>,

    /// The select routing faces to the primitive.
    pub select: String,

    /// The primitive's name, which the glTF bridge writes as
    /// `extras.vxl.name`.
    pub name: Option<String>,

    /// Whether the primitive writes `NORMAL`.
    pub normal: bool,

    /// The declared stream list; when absent, the material's list applies.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The vertex attribute writes.
    pub attributes: Vec<AttributeWrite>,
}

/// A whole meshing run, with the flags and profiles lowered in.
pub struct MeshRecord {
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
    pub materials: IdVec<BMeshMaterial, MaterialRecord>,

    /// The primitives by index; the implicit primitive lowers as an entry
    /// whose `true` select takes every face, so the table holds at least one.
    pub primitives: IdVec<BMeshPrimitive, PrimitiveRecord>,

    /// The loose files written beside the mesh.
    pub files: Vec<FileWrite>,

    /// The object's named properties.
    pub mesh_extras: Vec<ExtraWrite>,
}
```

Attribution stays in vxl, which maps each record element to its origin as it
lowers. A voxsmith error identifies the element it rose from, and vxl rewraps
the error to point at the flag or the profile entry.

voxsmith's entry point, `mesh`, runs in memory. It takes the record, the loaded
state, and the selected object, and returns the [document](#meshdoc). It calls
[vox-value-language](#vox-value-language) to parse, check, and evaluate the
program, then meshes the geometry, bakes the atlases, and encodes the images
through the png encoder its dependencies inject. Every voxsmith operation takes
that shape: it runs over voxcore and meshdoc types, reads no file, and knows no
format. vxl holds both ends of the run. It loads the state through voxconv and
saves the document through meshconv, which lands the document's files beside
the mesh. The slot and attribute writes resolve under meshdoc's material and
stream vocabulary, so voxsmith never learns the output format, and the bridge
maps that vocabulary onto the format's schema. The tests build a state and a
record by hand and never touch a file.

## meshdoc

meshdoc is the document every open phase writes into: a `MeshMain` holding
hierarchy nodes, objects of primitives, materials, textures, images, and
files, with every cross-reference a branded id the mutations check. The
[closed notes](#the-meshdoc-crate) describe the crate. What the phases need is
where each element of the [record](#one-record) lands:

- A material lands as a `MeshMaterial`. A slot write fills the modeled field
  its property names, a factor or a texture slot, under the names the
  `material` module fixes.
- An extras write lands as a `MeshProperty` on the material or the object:
  the JSON forms as the typed values, an image as a texture reference, and a
  written file as a file reference. The glTF bridge writes the properties
  under `extras.vxl.values`.
- A file write lands as a `MeshFile`, which meshconv writes beside the mesh.
  A png also lands as a `MeshImage` over the file when a slot or an image
  extra references it.
- A primitive lands as a `MeshPrimitive` of the object, its name written by
  the bridge as `extras.vxl.name`. A builtin attribute write fills a modeled
  stream, the normals or the vertex colors, and a custom write appends a
  `MeshVertexAttribute`. The UV streams it declares append in `texCoord`
  order, and a texture reference names its stream by id.
- The record's tables share `BMeshMaterial` and `BMeshPrimitive` with the
  document. The materials retain in table order and the primitives retain
  into the one object in table order, so a record id indexes the document
  unchanged.

Positions are in meters, Z-up, and the images stay encoded: the run encodes
each atlas once through the injected png encoder, and no bridge decodes one.
Where a written image embeds or lands loose is meshconv's write option, not a
document fact.

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

`mesh` lands, taking the record, the state, and the object to a
[document](#meshdoc) with geometry alone, through the kept greedy sweep. This is
voxsmith's half of the spine: `mesh` and the record exist from here on, and the
later phases fill in the rest of the document.

### 2. The environment

The effective palette binds its properties in atlas-texel order, and the
[computed values](value-language.md#computed-values) bind on request.

### 3. The evaluation

The entry point supplies the environment and the groupings, and `check` and
`eval` run the program over the [domains](value-language.md#domains).

### 4. The geometry

The kept greedy sweep learns the [merge rules](mesh.md#atlases) the run's values
set.

### 5. The atlases and streams

The four layouts shape onto the canvas, the UVs land at texel centers, and every
`texCoord` [derives](mesh.md#uv-streams).

### 6. The primitives

The selects [partition the faces](mesh.md#primitives-and-materials), and each
primitive carries its own attributes and streams.

### 7. The files

The png encoder grows grey, grey-alpha, RGB, and the transfer chunks, and the
file writers land the pngs and the JSON as the document's files.

### 8. The slots

The slot writers fill
[material fields and textures](value-language.md#material-slots), embedding a
value or referencing a written file, under the `material` names and the
encodings the modeled fields fix.

### 9. The extras

The extras writers land the named entries in their four forms as properties
of the materials and of the [object](mesh.md#palettes), and the bridge writes
them under `extras.vxl.values`.

### 10. The vertex attributes

The primitive writers land
[`COLOR_0` and the underscore attributes](value-language.md#vertex-attributes)
on the corners, as the vertex colors and the further vertex attributes, with
lower domains climbing in.

## vxl

### 1. `mesh-old`

The shipped `mesh` renames whole on both sides: vxl's subcommand, its module,
and its flag types, and voxsmith's `mesh` operation with its feature. The
renamed operation keeps serving the renamed command untouched, so a working bake
stands beside the rewrite until [the close](#the-close) removes both together.

### 2. The spine

A new `mesh` lands beside it, taking the input, the output, `--to`, `--from`,
and the selectors. It loads the state, builds a minimal [record](#one-record),
calls [`mesh`](#1-the-entry-point), and saves the document through meshconv.
The command runs end to end from here on, geometry only.

### 3. The flags

The full surface of [`vxl mesh`](mesh.md#options) lands in clap, each flag but
`--to` and `--from` lowering into its element of the record. vxl checks what
flags alone decide, and the entry point errors on an element it cannot mesh yet
and says which, so the whole surface lands ahead of the meshing work.

### 4. The profiles

The [schema](profile-language.md#schema) types land with serde, the
[built-ins](profile-language.md#built-in-profiles) embed, and `--profile` and
`--values-from` expand into record elements under the
[flag-beats-profile rule](profile-language.md#expansion). The checks fire at the
[stages](profile-language.md#loading) the profile language sets.

### 5. The cascade

`.vxlconfig` loads through [ty-preferences](#ty-preferences), and the track
closes with profiles reading through every layer.

## ty-preferences

ty-preferences reads `.vxlconfig` as it stands: its jsonc codec accepts comments
and trailing commas, and its loaders take the file name and the section key. The
layers load in application order, the user's `~/.vxlconfig` first and then every
directory from the git root down to the working directory. Outside a git
repository only the user layer loads. [Loading](profile-language.md#loading)
sets how profiles read through the layers, and vxl passes `.vxlconfig` and the
`mesh` key its envelope defines.

## The close

`mesh-old` deletes whole, the command, its flag types, and the voxsmith material
path only it still calls, taking the first [code deletion](#code-deletions)
with it; the second lands inside voxsmith's png encoder. A sweep follows the
delete: every `.rs` file the shipped implementation reached is checked for
remaining callers, and the files nothing reaches anymore go too. The sweep also
drops `MeshGeometry` and its mesher from voxsmith's public surface, because
nothing outside the crate reads them. Raw geometry stays a crate-internal seam
between the kept sweep and the [document](#meshdoc). The plan closes when the
[worked examples](examples.md) run as written.

## Code deletions

1. voxsmith's `MaterialSlot::OcclusionMetallicRoughness`: two
   `--write-material-slot-value` flags naming one value share the image, so the
   combined variant goes.
2. The png encoder's contract fixes 8-bit RGBA; the sized-to-value rule needs
   grey, grey-alpha, and RGB, and the transfer chunks want the png crate's
   `set_source_srgb` and `set_source_gamma`.

## Closed

The mesh crates and the paths through them landed ahead of the tracks above.
Each entry states what stands and where the built thing bent from the design.

### The meshdoc crate

`projects/utilities/meshdoc` holds `MeshMain<T>`, the state with an ext, and
its `MeshState` read side. The entities are hierarchy nodes forming a DAG with
roots, objects of primitives, materials, textures, images, and files. A
primitive holds positions, optional normals, tangents, and vertex colors, UV
streams by id, further vertex attributes as name, width, and flattened
components, and triangles of branded vertex ids. A material holds the
metallic-roughness set, alpha mode and cutoff, double-sidedness, ior,
transmission, and emissive strength as modeled fields, and named properties
for the rest. An image keeps encoded bytes, its own or a file's, and never
pixels. Every cross-reference is a branded id that the mutations check, and an
in-use error lists the referencing ids. `validate` states what the mutations
preserve, and `gc` compacts the pools and remaps the ext through `MeshExt`'s
hooks. The `check` and `material` modules hold the check vocabulary and the
property names with their defaults. The crate performs no side effects and
carries no `Dependencies` trait.

The design bent in two places. The document gained files: a `MeshFile` is a
named loose file that an image reads through or a property points at, so a
run lands its pngs and JSON in the document and meshconv writes them beside
the mesh. And the material model is fixed fields plus typed properties in
place of a slot list under a format vocabulary, so a bridge fills the common
case without a translation table and a value the model does not place still
has a typed home.

### The gltf-meshdoc crate

`projects/mesh-formats/gltf-meshdoc` converts between glTF 2.0 and the
document over the `gltf` crate. `from_gltf_file` loads a `GltfFile`, the JSON
root with its binary blob and loose files, into a `GltfMeshMain`, and
`to_gltf_file` writes one back, exactly for a loaded file, with
`GltfWriteOptions` choosing where the images go: as loaded, embedded, or
loose. `to_gltf_mesh_main` synthesizes the ext for a bare document. `GltfExt`
keeps what glTF has and the document lacks, keyed by entity id and following
the state through the hooks: the asset block, scenes, cameras, skins,
animations, morph targets, and the extras and extensions the model does not
place. The `codec` module goes to and from `.gltf` and `.glb` bytes, lists the
loose URIs a primary references, and frames GLB in the crate; the data URIs go
through injected base64 codecs bound behind `impl`. Axes convert between Y-up
and Z-up on both arrows. A material's or mesh's `extras.vxl.values` load as
properties and write back from them, a primitive's `extras.vxl.name` as its
name, and a property's file reference resolves against the loose files.

### The meshconv crate

`projects/utilities/meshconv` fronts the bridges. A `Format` marker per
enabled feature, `ReadFormat` and `WriteFormat` with their visitors, and
`read`, `write`, `load`, `save`, and `check_document_files` move a document
through a format over the caller's `Dependencies`, with the bare-state path
dropping the ext and the `ext` module's `_with_ext` pairs keeping it boxed. A
document travels as `MeshDocumentFile`s, the primary at the empty path and
each loose file at the relative path the primary references it by. A format
lists the loose paths its primary references, so a read fetches them beside
the primary and a write lands them beside the output. One format, `gltf`,
covers both containers, with `GltfWriteFormat` pairing the container with the
bridge's image storage. A new format adds a bridge crate and a feature here.

### The command tail

`vxl mesh` loads through voxconv, meshes into a document, and saves through
meshconv, with `--to` picking the container and `--texture-storage` where the
images go. vxl depends on meshconv and meshdoc and never on a bridge.

### The retired maps listing

Nothing emits `extras.vxl.maps` anymore. The writers that did went with the
format code voxsmith no longer holds, and
`--write-material-extra-image-value` is the deliberate replacement.
