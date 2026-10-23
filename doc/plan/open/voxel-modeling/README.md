# Voxel modeling plan

Status: **open**, drafted 2026-09-30. The [checklist](checklist.md) tracks both
phases.

## Goal

Claude Code builds a voxel model from a prompt such as "make a voxel chair with
oak and ornate jewels in the top". Claude writes a short TypeScript program
describing the model's shapes and materials, voxelizes the program into a voxj
document, and renders the document. Claude reads the renders and revises the
program until the model matches the prompt. Every step runs through `vxl`, so a
person can run the same loop by hand.

## The loop

1. **Author.** Claude writes a model file against the TypeScript API.
2. **Voxelize.** `vxl sdf-doc build` records the model as an `.sdfj` document.
   `vxl sdf-doc voxelize` samples the document into voxj and prints a report.
3. **Review.** `vxl object render` draws the review views onto one sheet.
   Claude reads the sheet beside the report.
4. **Revise.** Claude edits the model file and runs the loop again.

```sh
vxl sdf-doc build chair.ts
  --library materials
vxl sdf-doc voxelize chair.sdfj
  --voxel-size 0.025
  --report
vxl object render chair.voxj
  --profile review
  --to png
```

The review step decides the quality more than the authoring language does
because spatial reasoning is Claude's weak point. [Review](#review) covers what
the loop hands back.

## Reference

Four pages specify the pipeline:

1. The
   [TypeScript API](../../../../projects/utilities/sdfj-builder/docs/typescript-api.md)
   lists every call a model file can make, with its signature and its effect in
   a line or two.
2. The [modeling API](../../../../projects/utilities/vxl/docs/modeling-api.md)
   sets how vxl runs a model file and how the model lands on the grid.
3. The
   [sdfj format](../../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md)
   sets the document the builder writes and what each entry computes at a
   point.
4. [Model evaluation](../../../ref/sdf-doc/model-evaluation.md) sets how vxl
   builds the document and samples it onto voxels.

## Shapes

A shape is a signed distance function: negative inside, positive outside, and
zero on the surface. A voxel lies inside a shape when the function at the
voxel's center reads zero or less. Voxelizing reads only the sign, so a shape
needs no exact distance: smooth blends, twists, and noise displacement all work.
Where the distance is exact, a shell or an offset reads the distance to measure
thickness.

Coordinates count meters. A model holds no voxel size because sampling stays a
separate step: `vxl sdf-doc voxelize` takes the voxel size or a resolution from
its flags. A box with corners on multiples of the voxel size fills exactly the
cells between them. Models run from 16 to several hundred voxels across. A
sphere under a radius of about 3 voxels comes out as a plus sign. Boxes aligned
to the grid carry the smallest forms.

## Steps

A model holds an ordered list of steps over a grid. A later step wins wherever
two steps reach the same cell. An edit usually changes or appends one line
because the list reads in building order: the blocking out first, then the
detail. Each step carries a name that the report and errors use.

## Materials

A material holds voxj properties as key/value pairs. voxj's glTF vocabulary
gives the named properties, and any other name adds a custom property. A step
takes a library material, a custom material, or a pattern. A pattern picks one
material per cell and never blends colors. The palette then stays small. A
library name means one look in every model. vxl ships the `materials` library,
and a `.vxlconfig` can define more libraries through the cascade.

A pattern reads coordinates in the frame its shape was built in, and grain
therefore runs along each leg however the leg turns. Painting belongs in the
model because a finished voxel remembers neither its step nor its frame. The
palette the model writes feeds `vxl object mesh` and `vxl object render` with no
extra step.

## Output

`vxl sdf-doc voxelize` writes one voxj document beside the `.sdfj` by default. A
model without parts writes one object under one root node. The model's
`[0, 0, 0]` lands on the root node, and a chair built centered on x and z with
its feet at `y = 0` stands on its pivot. The root node's scale carries the voxel
size, and the placed document therefore measures meters.

A model can group its steps into parts, and nested parts form a hierarchy. Each
part voxelizes on its own grid into its own object under a node at its pivot,
and turning the node turns the part about its joint. `--select` can then pick a
part by its path. A step reaches only its own part's cells, and parts can
overlap. A part can sit in several lists, and an `offset` moves each place. The
default `--frame world` samples every place on one lattice, and `--frame local`
voxelizes a part once for all its places to share. `--flatten nodes` gathers the
parts' objects under one node, and `--flatten objects` merges the parts back
into one object.

## Review

`vxl sdf-doc voxelize --report` prints a report because Claude reads numbers
more reliably than pixels. Each step takes one line with the cells the step
wrote, the cells it keeps, and how many kept cells show. The step's size in
cells and bounds in meters follow. The report also splits the model into
face-connected pieces. A buried jewel, a floating leg, and a hand detached from
its arm all show up in the report before any render.

The renders come from a built-in `review` render profile: orthographic `front`,
`right`, `back`, and `top` views for checking proportions, and the perspective
`hero` view for the look. The profile tiles the five views into one contact
sheet. One image read then covers every side.

```sh
# chair.png, the sheet of the five views
vxl object render chair.voxj
  --profile review
  --to png
```

## Skill

A skill teaches Claude the loop because nothing else tells Claude the builder
exists. vxl installs the skill as one `SKILL.md`:

```sh
vxl integration skill vxl-model install claude
```

`SKILL.md` holds the workflow, the TypeScript API, and the modeling API with its
example model. One file loses nothing because Claude reads both APIs every pass.
`install claude` writes `.claude/skills/vxl-model/SKILL.md`, and
`install agents` writes it under the `.agents` that Codex reads. The install
works at the git root, or under the home directory for every project with
`--user`. `vxl integration agents-link` links `.claude` to `.agents` so both
agents share one folder. `SKILL.md` records the vxl version that installed it.
Upgrading vxl takes a reinstall.

`vxl integration` gathers the commands that set up other tools to work with vxl.
`vxl integration completion <shell> print` takes over from `vxl completion`.
`install` writes the completions where the shell loads them. Every binary in the
workspace takes its completion commands from ty-clap. `--help` lists the shells
and skills each command takes.

The skill walks Claude through the workflow: pick a voxel size, write real sizes
in meters, block out with boxes, voxelize at the chosen size, review, fix the
proportions, then detail and paint. At 2.5 cm per voxel, a seat 45 cm up sits 18
voxels up. The skill also says what each pass checks in the report and the
renders. The skill chains the three commands into one line because a pass that
skips the build voxelizes a stale `.sdfj`.

## Phases

1. **Pipeline.** A TypeScript builder records the model as an `.sdfj` document,
   and vxl does the rest in Rust. `vxl sdf-doc build` runs the builder under
   Node, Bun, or Deno. vxl embeds the builder, so an installed vxl needs no
   checkout. `vxl sdf-doc voxelize` checks the document and samples it through a
   voxsmith operation that follows model evaluation. The `materials` library,
   the `review` profile, and the skill ship inside vxl. Trials then record which
   shapes and patterns Claude reaches for and where the loop fails. The
   [prompts](trials.md#prompts) run from a chair to a small city and the lower
   deck of a pirate ship.
2. **Follow-ups.** The trials set the second phase. The report and a contact
   sheet hand back more per pass, and the library's colors read truer. New
   options keep paint on its target. New calls draw curves, text, and brickwork,
   and parts turn at any angle. Two more trial rounds measure the changes.

## Waiting for a reason

Phase 1 left these candidates open, and the trials asked for them rarely or
never:

1. Patterns that take patterns as entries, for knots over grain or moss over
   cobbles
2. A `hueShift` option on `shades` that turns the darks' hue one way and the
   lights' hue the other
3. A step that runs a part's steps on another part's grid
4. A `typescript` library group whose sources vxl builds before the model
5. Review sheets that pick their views per model, such as a left view or no
   back view behind a room, for round 7's finding 127

### Voxel space

These candidates work on the voxel grid, while a model builds in signed
distances. They wait for one approach to editing the grid, such as a family of
`vxl object` and `vxl node` commands, rather than a fix at a time:

1. A step that picks darker shades in crevices and lighter ones on edges. Review
   renders shade crevices already, but an exported mesh keeps only the colors
2. A step that hollows the finished grid without the walls a union's shell
   leaves
3. A `vxl node settle` that seats one placed object on another for finding 11.
   Round 7's later passes seated most floating details within one pass
4. A `vxl object` command that draws a face-connected line of voxels between
   two points for finding 130. Only the guitar's strings asked for one
5. A prune of small pieces for finding 24. Round 7's crumbs and intended pieces
   overlap in size, and its broken strings, handles, and knobs are as small as
   its crumbs, so a prune by size deletes details that need fixing

### Beyond realtime SDF

These candidates need more than the distance at one point. A realtime SDF
renderer evaluates every call per point, so they wait for one approach that
solves them ahead of time rather than a fix at a time:

1. A `settle` option on `add` that moves the shape toward a side until it
   touches the shapes before it, for finding 11. vxl solves the move over
   whole shapes

## Decisions

1. Shapes are signed distance functions sampled at voxel centers. Only the sign
   decides a voxel.
2. Combinators build shapes. An ordered list of steps applies the shapes to a
   part's grid, and the later step wins.
3. Claude writes TypeScript against a typed builder and never raw per-voxel
   loops. `set` fills a cube at each point for detail too small for a shape.
   TypeScript gives the model variables, functions, and loops. Node 24, Bun, and Deno all run
   TypeScript with no build step.
4. The operator set stays closed, with no custom per-voxel code in the model. A
   closed set keeps the output vetted and lets vxl evaluate every operator in
   Rust.
5. Patterns pick among materials and never blend colors.
6. The TypeScript API and the modeling API serve the agent writing a model, and
   the sdfj format and model evaluation serve the implementations. The agent
   loads only the two APIs.
7. Mesh CSG stays out. Building a `.glb` for `vxl mesh-doc voxelize` works today
   but cannot write per-voxel patterns and loses detail through the mesh.
8. The builder only records. A model's calls become an `.sdfj` document, which
   vxl checks and samples in Rust. Everything past the TypeScript lives once.
   The sampling reuses `mesh-doc voxelize`'s sizing, profiles, and encoding.
9. The loop uses only vxl. `vxl sdf-doc build` wraps the JavaScript runtime a
   flag or profile picks and assumes the runtime is installed.
10. A model holds signed distance functions in meters and no voxel size.
    Sampling stays separate. `vxl sdf-doc voxelize` follows
    `vxl mesh-doc voxelize`: the output path, the sizing flags with their
    defaults, `--frame`, `--fill-mode`, the encoding flags, and profiles in
    `.vxlconfig`. The lattice anchors at the origin. The root node's scale holds
    the voxel size. Every length a model writes reads in meters.
11. Each part voxelizes on its own grid into a voxj object. A step reaches only
    its own part's cells, and parts can overlap. A turned joint then shows each
    part whole.
12. A material holds voxj properties as key/value pairs. The glTF vocabulary
    gives the eight named properties, and every palette binds all eight. A
    material leaving one out takes glTF's default, except that `metallic`
    takes 0. Any other name adds a custom property of the kind its value takes.
13. The report prints plain text for an agent to read. Every number sits beside
    its label, with bounds in the meters a model writes. JSON would double each
    pass's tokens, and a table's header would hold the labels away from the
    values.
14. vxl ships the skill. `vxl integration skill vxl-model install <target>`
    writes it under `.claude/skills` or `.agents/skills`. `print` prints it to
    stdout. The installed skill teaches the API of the vxl that installed it.
15. In every binary of the workspace, `integration` gathers the commands that
    set up other tools. `integration completion <shell> print` takes over from
    `completion`. `integration completion <shell> install` writes the
    completions where the shell loads them. ty-clap holds both for every binary.
16. The SDF crates follow the voxel and mesh families. The family keeps that
    structure even with `.sdfj` as its one format. The crates stay thin because
    voxsmith does the work.
    1. `sdfcore` at `projects/utilities/sdfcore` holds a model's state as tables
       indexed by branded ids
    2. `sdfj` at `projects/sdf-formats/sdfj` holds the `.sdfj` types.
       `sdfj-codec` reads and writes `.sdfj` documents over those types.
       `sdfj-sdfcore` converts between a document and the sdfcore state
    3. `sdfconv` at `projects/utilities/sdfconv` reads and writes SDF formats
       through the sdfcore state
    4. voxsmith evaluates and samples the sdfcore state under an `sdf_doc`
       feature beside `mesh_doc`
    5. `sdfj-builder` at `projects/utilities/sdfj-builder` holds the builder as
       a TypeScript package with the TypeScript API. vxl embeds the builder's
       `.ts` files and the TypeScript API, and holds the skill's workflow and
       the modeling API
17. A library shares named entries between models. A library holds an `.sdfj`
    document whose `names` table maps names to the document's entries.
    1. The builder writes a name for each of the model's named exports. Every
       built document can then serve as a library. A named export holding an
       array errors
    2. A model reads library entries through `lib`, with `mat` for the
       materials. Using an entry copies the entry and every entry it references
       into the model's document at fresh indices under the same names. The
       document then stands alone
    3. `sdfDoc.build.libraries` in `.vxlconfig` defines libraries by name.
       `files` holds each library as an `.sdfj` `path`, and `embedded` holds
       each library as an inline `document`. A layer replaces an earlier
       layer's library of the same name in either group. vxl's built-in layer
       embeds `materials`
    4. A build profile's `libraries` and `--library` list libraries by name.
       When two listed libraries hold one name, the later library wins. A build
       with neither reads no library
18. A step or a part can sit in several lists, as a voxj node can sit under
    several parents. `--frame world` moves each
    place's shapes by its offsets before sampling and keeps every part on one
    lattice. `--frame local` voxelizes each part once, and every place shares
    the part's object. Unique names within each list keep every voxj path
    unique. The `.sdfj` document mirrors voxj with `objects`, `nodes`, and
    `rootNodes`.
19. Phase 2 builds the follow-ups the later rounds kept asking for. Phase 1's
    other candidates wait for a reason.
20. sdfj suits a realtime SDF renderer and never depends on the voxel grid.
    Every call works at one point and costs little there. Candidates that need
    the grid or a solve beyond one point wait for an approach of their own,
    which may arrive as an extension.
21. A name appears once, where its thing is made. A model refers to the thing
    by value, and the `.sdfj` and `.voxj` documents by index. Names label
    errors and reports. A `names` map lists only exports.
