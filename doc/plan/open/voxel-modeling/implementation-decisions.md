# Implementation decisions

Code-level choices a reviewer of the Rust would want explained, recorded as they
land.

## S1. Document

1. sdfcore holds a model whole and never edits it. `SdfState` holds public
   `IdVec` tables, and `SdfMain::new` validates a state once. sdfcore has no
   retain, release, gc, or ext because nothing in Rust edits a model and one
   format leaves no foreign state to carry. A shade and its `shades` entry
   reference each other's tables, and a retain per entry would have to order
   the two. Validating whole tables avoids the order
2. Validation covers the format's rules, finite numbers, and unique property
   names and `json` keys. A state built in Rust can hold a NaN, and serde_json
   would write the NaN as `null`. The arguments' values wait for S5's checks
3. Numbers stay `f64`, counts and seeds included, because S5's checks test
   whether a count is whole. A shade's `index` reads as a whole number in the
   format, and sdfcore holds the index as a `u32`
4. Entries reference each other by `U32Id`. The bridge errors on a wire index
   past the `u32` id space, as voxj-voxcore does
5. A step's material or pattern is an `SdfStepMaterial` in sdfcore. sdfj
   mirrors the wire with two optional keys, and the bridge errors unless
   exactly one holds an index
6. `SdfEntryId` displays as `table[index]` with the wire table names, so
   sdfcore's errors point into the document. The bridge starts its own errors
   about one entry the same way
7. sdfj's variants mirror sdfcore's field for field. A reference field drops
   its `_id` or `_ids` on the wire, and `shape_id` reads `shape`. An optional
   key holding `null` fails to read
8. `SdfjMap` takes its value type. Material properties and `json` objects share
   one ordered map that rejects a repeated key
9. An `int` or `json` property value writes as an adjacently tagged
   `{ kind, value }`. The plain forms parse untagged
10. sdfj's types error on writing a NaN, an infinity, or a repeated object key.
    serde_json would otherwise write a non-finite number as `null` and keep both
    keys. sdfj-codec's encode returns a `Result` because a hand-built `SdfjFile`
    can hold any of the three. sdfj-sdfcore's byte writers `expect` success
    because `SdfMain` validation rules all three out
11. `SDFJ_VERSION` lives in sdfj. sdfj's parse takes any `u32`, and the bridge
    checks the version on read
12. sdfconv keeps meshconv's format markers and visitors but moves bytes instead
    of a file list and has no ext module. An `.sdfj` document is one file, and
    its state carries nothing past sdfcore's. `SdfjSerialization` picks compact
    JSON, the default, or pretty-printed JSON
13. The bridge's tests round-trip the chair, the forest, and a fixture holding
    one entry of every kind with every optional key

## S2. Builder

1. The builder is the TypeScript package `sdfj-builder` in
   `projects/utilities`, apart from vxl for a later npm release. The
   TypeScript sits in `ts/`. `main.ts` takes the model path and the output
   path and reads `libraries.json` from its own directory
2. The folder is also a crate with its Rust in `src/`. `SDFJ_BUILDER_FILES`
   embeds every file a run needs because a published vxl can include only
   files inside its own directory. A test checks the list against the folder
3. `deno.json` sets `nodeModulesDir` to `none` because Deno otherwise expects a
   `node_modules` beside the `package.json`
4. Each module holds one section of the modeling API. One export per file would
   split the API into about sixty files of a few lines each
5. The builder checks every argument against the type the modeling API
   declares, because a runtime strips a model's types without checking them. A
   value the document cannot hold then errors at the call that made it. Ranges
   and whole numbers stay vxl's checks. `shades` checks its count because it
   returns that many materials, and a boolean checks for a first shape because
   that shape picks the table
6. A value holds its entry in a private field, which keeps each class distinct
   to the type checker. Vectors, lists, properties, and `json` values copy at
   the call, and no value changes after its call returns
7. `globals.d.ts` declares each global as the type of its export from `api.ts`.
   `putApiInScope` assigns the globals through `Pick<typeof globalThis, ...>`,
   and a name missing from either file fails the type check
8. `deno.json` turns on `verbatimModuleSyntax` and `erasableSyntaxOnly`. With
   both on, every file Deno checks also runs under Node's type stripping
9. The builder writes compact JSON and a trailing newline, the form sdfj-codec
   writes by default. `jsonText` keeps the sign of `-0` and errors on a value
   without a JSON form instead of writing `null`
10. An error escapes uncaught. Each runtime prints the message and a stack whose
    frames include the model file's line
11. The tests run on `Deno.test` and `node:assert/strict`, and the builder stays
    free of dependencies. `test/chair.ts` and `test/forest.ts` hold the sdfj
    format's examples, and their tests compare the written documents with the
    format page's
12. The builder holds no material names. `mat` types every name as a
    `Material`, and the libraries vxl passes in decide which names it reads

## S3. Build command

1. sdfj-builder holds each runtime's program and arguments in
   `JavaScriptRuntime` and the libraries' path in `SDFJ_BUILDER_LIBRARIES_PATH`.
   vxl's code holds no path inside the builder's directory
2. `--config` points Deno at the builder's `deno.json`, which keeps a config
   beside the model out of the run. Deno's write access covers every path
   because Deno's permission lists split a path on commas with no escape
3. vxl's `Dependencies` gains `CreateTempDir` and `RunProgram`. The runtime
   inherits vxl's standard streams, which carry the builder's messages and the
   model's stack unchanged
4. The builder writes the document inside the temporary directory. vxl then
   writes the output through `WriteFile`, which creates missing directories for
   every command's output
5. A stand-in for `node` tests the run because `cargo test` runs without a
   JavaScript runtime installed. At S3, Node 24.15 and Deno 2.9 ran the builder
   through vxl by hand. Bun was not installed

## S4. Shapes

1. voxsmith's `sdf_doc` feature holds `SdfShapes`, which prepares every shape of
   an `SdfMain` once and evaluates a 3D shape's distance and frame position at
   a point. A 2D shape evaluates only through an `extrude` or a `revolve`
2. `SdfShapes` expects arguments that pass model evaluation's checks, which land
   at S5. Until then an argument a check rejects can panic with the check's
   rule
3. Every shape's box lands at S4 because a bend reads its child's box. A bend
   reads the box before rounding, which keeps a bent shape the same at every
   voxel size. S5 rounds the boxes and clamps the distances to them
4. The noise lands at S4 because `displace` reads it. The hash chains the
   lowbias32 mixer over the seed and the coordinates. A test measures `sigma1`
   again over 200 thousand points
5. The pyramid formula measures only to the slanted faces. Inside near the base
   it reads too deep, and below the base it measures to the nearest edge. The
   pyramid measures to the base square at or below the base plane and takes the
   nearer of the faces and the base inside
6. A lathe closes its outline with the outline's mirror image across the axis.
   An edge along the axis would read each point near the axis as near the
   surface, and a `shell` would keep a rod down the middle
7. A cut torus measures past its end to the end point directly. The capped
   torus formula takes the square root of a difference, which rounding can push
   below zero on the ring
8. A polygon edge with no length measures to its one point. A lathe's outline
   repeats a point that lies on the axis
9. Mirror copies run in binary order with x as the low bit, and repeat copies
   run with x outermost. A tie between copies goes to the earlier copy, which
   sets the frame position
10. glam's `length` sums the squares in coordinate order, as model evaluation's
    arithmetic asks
11. Each shape's test checks a grid against a reference computed another way:
    distances to segments, arcs, and triangles, a box's clamped point, a ternary
    search over a round cone's spheres, or a dense sampling of an ellipse. Every
    point a shape covers lies inside the shape's box

## S5. Steps

1. `sample` runs the checks, settles the voxel size, and runs each part's steps
   over its grid. `SdfSampling` holds the places depth first and the grids they
   index. A cell holds its material or pattern and the step that last changed
   the cell. For S8's report, each step records the count and the box of the
   cells it wrote
2. `sdf_doc` and `mesh_doc` share `GridResolution`, `ResolutionReference`, and
   `VoxelFrame` from voxsmith's `utilities`, so neither feature turns on the
   other. `ResolutionReference::side` measures a reference's side for both
   voxelizers
3. A 3D shape's box rounds out on its place's lattice: the box moves by the
   place's shift, rounds, and moves back. Only a 3D shape's distance clamps to
   its rounded box because a 2D profile's plane has no lattice to round to
4. The fields evaluate their children through `ShapeSource`. `SdfShapes`
   evaluates the formulas alone, and `LatticeShapes` clamps every level to the
   rounded boxes
5. A check error starts with the path of part names and the step name of the
   first step that reaches the failing entry. An error about an entry no step
   reaches starts with the entry's table and index instead
6. The checks that read boxes run after `SdfShapes` prepares the shapes.
   Preparing a bend reads its shape's box, so the check for a bounded bend shape
   runs earlier and decides from the document alone whether a shape has a box
7. The check reads a bend's half turn as the bent shape's whole span: the shape
   spans at most `pi * radius` along `along`
8. A polygon's check counts repeated neighboring points as one point. Two edges
   that are not neighbors cross where they meet, and two neighboring edges cross
   when they fold back along each other
9. The material checks land with S6's material properties

## S6. Materials

1. `resolve_materials` turns every material into `SdfMaterialProperties` before
   the sampling: the named properties in linear light or at their defaults, and
   every custom property the model's materials set. A material that leaves a
   custom property out holds the kind's empty value for S8's palette
2. A cell holds the material its step picked. A pattern picks from the
   evaluation that decides whether the step writes the cell
3. A `coat` or a `set` pattern reads the frame position a primitive returns:
   the cell's center minus the place's shift. A part placed twice under
   `--frame world` then patterns every step alike at both places
4. A hash picks material `hash mod n`. `cells` and `speckle` add a channel
   coordinate to draw independent numbers for one cube or cell. A seed left out
   reads 0. A pattern's `fbm` sums the 4 octaves `displace` defaults to
5. `shades` converts through Bjorn Ottosson's published matrices. A step of 0
   skips the conversion because the matrices round-trip only to about 1e-8. A
   base channel at 0 or 1 would otherwise come back outside the gamut
6. The checks hold `spread` above zero. A zero spread repeats the base, and a
   negative spread runs the shades from lightest to darkest
7. The named ranges come from meshdoc's material vocabulary. `normalScale` and
   `alphaCutoff` count as custom properties because the modeling API leaves
   them out
8. The kind check reads every material in the document, whether a step reaches
   the material or not

## S7. Libraries

1. sdfcore holds each names table as `(String, U32Id)` pairs because the id
   brands derive no traits. A generic struct deriving `Clone` would need its
   brand to implement `Clone` too
2. `names` lists its keys in the document's table order. `parts` maps to
   `nodes` because a part writes one node
3. Every table and names map can stay out of a document. serde reads a missing
   table as empty and skips an empty one on write. The Rust types keep every
   table. The builder converts at its edges: `sdfjJson` drops the empty tables
   it writes, and `sdfjDocumentFromJson` fills in the tables a library lacks
4. The builder writes the named exports in name order after the default export
   because a module namespace lists its exports by name. The builder writes
   each names table in name order too
5. A `WeakMap` holds the library names of each value read from a library. Every
   document that writes the value writes those names too. The writer gathers
   the library names and the export names after the walk
6. `lib` and `mat` read an entry through a getter on first use. The builder
   never reads a library's unused entries. A library node that fails to read as
   a part errors only when a model uses the part
7. The builder reads a library part's steps before its child parts. A copied
   part whose list interleaved steps and child parts keeps its node and object,
   but its entries can take other indices
8. vxl hands the builder `libraries.json`, an array of each library's `name`
   and `document`. The name lets the builder's errors point to the library
9. vxl loads the cascade's libraries only when a build lists one. sdfj-sdfcore
   checks each listed library before the builder runs
10. vxl embeds its built-in layer as the JSONC text `built_in.vxlconfig`. The
    text parses through the code every `.vxlconfig` layer takes. A comment
    beside each material shows the material's name
11. Nothing prints a library's `description` yet
12. A vxl test samples every built-in material with its default `shades`

## S8. Voxelize command

1. voxsmith splits a run in three: `sample` makes the grids, `to_vox_main`
   writes them as a `VoxMain`, and `report` writes the report's text. vxl
   samples a second time under the world frame only for `--report` beside
   `--frame local`
2. `SdfSampling` records the frame the grids were sampled under.
   `to_vox_main` errors on flattening local grids, and `report` errors on local
   grids
3. `FillMode` moves to voxsmith's `utilities` beside `VoxelFrame`, and the new
   `FlattenMode` joins them. vxl's help for `--fill-mode` and `--flatten` reads
   for meshes and models alike
4. `mesh-doc voxelize` takes `--flatten` too. A mesh's root nodes stand in for
   the root parts. The mesh voxelizer merges the objects before it builds the
   palette, so a material only an overwritten cell held stays out
5. Under `--frame world` and `--scale bake`, a mesh node's scale drops its
   placement's mirroring because the world grid already holds it. Before S8, a
   mirrored placement landed mirrored twice
6. A component of `f` counts as whole within `1e-9` of a whole number because
   the division by `g` rounds
7. An object takes its origin from its first place. A later place with another
   `f` places the object through a `voxels` node at the difference. Only a
   document whose nodes share an object across pivots meets this. The objects
   of one place at one offset share one `voxels` node
8. A step line shows the step's name alone because the part lines above it give
   the path. Only the piece lines take paths
9. vxl's `Dependencies` gains sdfconv's traits because vxl reads the `.sdfj`
   document through sdfconv. vxl takes voxsmith's `sdf_doc` feature
10. Both voxelize commands share `GridResolutionOptions` and read a profile's
    `resolution` and `voxelSize` through `profile_grid_resolution`

## S9. Review profile

1. `review` imports `front`, `right`, and `top` beside `hero` and sets only the
   three sides' projection
2. The PBR Neutral tonemap never writes 255. Under `studio`, a white base color
   can also render dimmer than a warm near-white one, so neither 255 nor a
   white material can stand for white in a test. A test instead checks that
   under `review` each library material's lightest default shade renders
   brighter than the material at every pixel. A shade at the tonemap's ceiling
   would render no brighter. The library passes unchanged

## S10. Integration commands

1. Each binary declares its `Integration` enum in `main.rs` beside `Command`
   because the binaries share no crate that depends on clap
2. Each command path runs nouns then a verb, as `vxl object voxels flip` does.
   `integration completion print` prints the completions, and
   `integration skill print` prints the skill
3. Each `print` command's help ends with the commands that install the printed
   file. The tyt binaries share `completion_install_help` from tyt-common
   because the help needs no clap. vxl keeps a copy because vxl depends on no
   tyt crate

## S11. Skill

1. The modeling API moves to vxl's `docs/` and the workflow to vxl's
   `skills/vxl-model/`. A published crate embeds only files inside the crate's
   directory
2. `AgentSkill` in vxl's library builds each skill's `SKILL.md`. The
   frontmatter's `name` takes the skill's command-line value. `metadata` records
   the vxl version
3. A test checks that each link in a skill reaches one of the skill's headings
   or an `https://` URL. The same test fails when two headings share an anchor
   because the workflow and the modeling API land in one `SKILL.md`
4. The workflow writes the pass on one line because Claude runs the line as
   written. A flag per line would split the pass into separate shell commands
5. The sdfj format moves to `docs/sdf-json-file-format.md` with the title SDF
   Json File Format. The file name and title follow voxj's
   `docs/voxel-json-file-format.md`

## S13. Round 1 fixes

1. `studio`'s hemisphere takes a white sky over a #404040 ground. The hemisphere
   alone colors a metal because a voxel face misses the key light's highlight. A
   blue-gray sky turned gold olive and steel navy
2. `grain` hashes the frame position's distance from the line along `axis`
   through the frame's origin. A shape built around the origin and moved by
   `translate` centers its rings because a pattern reads the frame from before
   the transforms
3. `box`, `boxFrame`, and `rect` keep `min` and `max` as their parameter and key
   names. vxl sorts the corners per axis when it builds each shape's field. The
   checks read the sorted size. The document keeps the corners as the model
   wrote them. `arch` keeps ordered corners because its `max` sets the height
   above the spring line
4. Every binary's `Cli` takes `#[command(version)]`, which prints the binary
   crate's version. The tyt-meta `main.rs` template carries the attribute
5. `review` sets `glow`'s three bloom values itself because a profile imports
   only views and lights. Stacking `review` with `glow` errors as any element
   two profiles set does. `--bloom-strength 0` turns the bloom off
6. The bloom threshold reads an emission's brightest channel. Luminance weighs
   green ten times blue, so a pink at strength 5 barely bloomed beside a green
7. The library shade test renders `review` with the bloom off because a material
   and its lightest shade share one emission and one halo
8. In the light group, `ember` and `lava` drop to strength 1 and `flame` to 1.5
   over a darker #9A5A1E base. Each core then keeps its hue under `review`.
   `glow` keeps strength 3 because its near-white color whitens without losing
   its blue
9. The library's `grass` takes #6A8547. The more saturated #5A8F3A read as a toy
   green beside the other library colors

## S17. Review sheet

1. A profile holds views, images, and a rig. Each image draws one view or
   tiles several into a sheet. A view no image draws never renders, so a
   profile can hold views only for import
2. An empty image entry draws the view of its name, so
   `"images": { "hero": {} }` and `--image close` each declare a whole image
3. A view a `--view-*` flag creates takes an image of its name until the run
   gives an `--image*` flag. A view a flag creates or changes that no image
   draws errors, so no hand flag drops silently
4. A sheet's tiles take an 8 pixel #808080 border by default because the gray
   stands out on light and dark backgrounds
5. A part's `rotation` sets its node's rotation. The part voxelizes unrotated.
   The render's `--rotate` rotates the same node, so a preview copies into the
   model file one axis rotation at a time. One pose per part covers the four
   test models posed by hand
6. An axis rotation takes one axis letter and degrees, as a shape's `rotate`
   does. A list of them composes on the parent's fixed axes in order

## S18. Library colors

1. `shades` defaults to a spread of 0.05. Side-by-side slabs under the review
   rig showed 0.08 staining steel and gold, and 0.04 flattening ebony
2. `glow` drops to strength 2.75 because at 3 a 0.05 step left its lightest
   shade at the tonemap's ceiling. The review renders show no difference
3. A lighter shade errors when sRGB holds it only with less of the base's
   chroma, because the faded shade read cream on gold. A loss under 1e-6 passes
   because a gray's chroma comes from float error
4. Both shade errors end with the largest spread at which every shade fits. The
   error rounds the spread down, which keeps the given spread passing. The
   chroma check stays exact because the error states the fix
5. `oak` takes #9D7D51, `walnut` #563F31, `rope` #AE946D, and `grass` #5E8445.
   Oak loses chroma to stop reading orange. Walnut turns from pink toward brown,
   rope from khaki toward tan, and grass away from `moss`
6. The metals wait for the rendering follow-ups' reflections. A metal face
   toward the camera reflects the review rig's half-gray horizon, and neither a
   brighter base color nor a lighter ground brought gold back from olive
7. The dark woods, stone, and bark keep their colors because their patterns
   show on every lit face under the review rig at the default spread. Most of
   round 6's near-black cases came from iron. Iron waits with the other metals

## S20. Spills

1. The last `add` that fills a cell owns it. A `carve` that empties the cell
   clears the owner, and `set`, `paint`, and `coat` leave it. A renderer
   reading the shapes at a point finds the same owner
2. A part keeps owners only when one of its steps takes `onto`, so other parts
   use no more memory
3. The builder checks `onto` at the call and again in the part's list. Model
   evaluation checks it once more for documents other tools write
4. `coat` tests at most 6 points per cell, one `depth` meters out toward each
   listed side. A gap narrower than `depth` under a shape beyond it takes no
   coat
5. vxl errors on a `depth` under one voxel because a shorter one coats some
   cells of a curved or off-grid surface and skips others
6. A `set` point holds no shape, so a coat reads through it
7. `depth` moves to a positional argument because the API takes each required
   value as one

## S21. Curves

1. `spline` closes into an outline without `width` and runs open as a stroke
   with it, so one call covers both
2. The curve runs centripetal Catmull-Rom through every point, which never
   kinks or loops within a span however unevenly the points lie
3. Evaluation splits each span into 16 segments and measures the result as a
   `polygon` or a `polyline`. The split ignores the voxel grid, so a renderer
   or a mesh export draws the same curve
4. `sweep` keeps the profile's v as near `up` as the curve allows, so a helical
   rail or ramp stays level. A rotation-minimizing frame never fails but rolls a
   rect profile along a helix. A curve that runs along `up` errors until the
   model passes another `up`
5. `sweep` takes one profile scale per point because a body that bulges
   mid-length needs more than an end scale. The scale runs smoothly between
   points. A curve that takes it below 0 errors
6. `helix` climbs a total height over a number of turns because a stair knows
   its floor-to-floor height. It returns points and leaves sdfj unchanged

## S22. Meters only

1. The `.sdfj` `version` stays 1. A document that leaves out a length now
   errors on the missing key
2. A `cells` border holds its material and its width in one object, so a border
   cannot come without a width
3. `set` takes its size before its material because the points and the size
   describe what fills, as an `add`'s shape does
4. A `set` cube takes a cell whose center lies on its min face and leaves one
   on its max face. A size of k voxels then fills k cells per axis wherever the
   point lies
5. `speckle` hashes the cube of its size that holds the cell's center. A size
   of one voxel picks per cell as before
6. tyt-assets' models write the old defaults out at their voxel sizes. They
   voxelize byte for byte as before, so their `.vmax` files stand

## S23. Content interning

1. Entries match on their compact JSON text. Two entries that list the same
   keys in another order, or write `0` and `-0`, stay apart
2. `--intern false` or a build profile's `"intern": false` turns interning off.
   vxl then passes `--intern false` to `main.ts`, and otherwise passes no flag
3. Equal values share one entry, and their names can reference one index
4. The sdfj format's chair writes its `shades` entry and its three shade
   materials once each
5. tyt-assets' models voxelize byte for byte as before. A model of 400 parts
   that rebuild 60 profiles each shrinks from 3.9 MB to 51 KB, and its build
   takes 0.19 s either way, so multithreading waits

## S24. Build variables

1. A variable holds any JSON value. `--var` reads the text after its first `=`
   as JSON, so a string value takes quotes
2. Each `--var` replaces the profile's variable of its name and keeps the
   others. A name given to `--var` twice errors
3. vxl writes the variables to `vars.json` beside `main.ts`, as it writes
   `libraries.json`
4. `vars` types its values as `any` because each model knows the shape of its
   variables
