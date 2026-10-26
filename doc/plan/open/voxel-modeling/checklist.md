# Checklist

The [README](README.md) holds the design. Each step is checked off as it lands.
Code-level choices go in an `implementation-decisions.md` beside this file.

## Ground rules

- The builder only records the model. Every check the `.sdfj` document can carry
  runs in vxl. Every voxj document vxl writes passes `vxl vox-doc validate`.
- The
  [TypeScript API](../../../../projects/utilities/sdfj-builder/docs/typescript-api.md),
  the [modeling API](../../../../projects/utilities/vxl/docs/modeling-api.md),
  the
  [sdfj format](../../../../projects/sdf-formats/sdfj/docs/sdf-json-file-format.md),
  and [model evaluation](../../../ref/sdf-doc/model-evaluation.md) lead. A
  change to the commands' surface or behavior lands in them in the same step.
- The builder has no dependencies and no install step. The builder runs under
  Node 24, Bun, and Deno and imports only `node:` modules and the builder's
  `.ts` files.
- `deno check`, `deno lint`, and `deno fmt --check` pass over the builder before
  a step stages. The builder's tests sit in `*.test.ts` files beside the code
  they cover and run under `deno test`. The pre-commit hook runs all four on a
  commit that touches the builder.
- A new crate starts at `0.1.0` with
  `[package.metadata.workspaces] independent = true` and joins the workspace
  members and `[patch.crates-io]`.
- Trial notes go in `trials.md` beside this file. A trial model that turns out
  well joins the builder's `examples/`. Renders stay out of git.

## Phase 1: pipeline

The builder records models as `.sdfj`, and vxl voxelizes and renders them.
Trials then show which operations Claude reaches for and where the loop fails.

- [x] **S0. Open questions.** Settle the README's open questions and fold the
      answers into the plan and the reference pages.
- [x] **S1. Document.** A reference page for the `.sdfj` document beside the
      other two, with the entry the builder writes for each call in the modeling
      API. The document follows voxj: tables whose entries reference each other
      by index hold every value a model can share, including steps, objects, and
      nodes. The crates the [crate layout](README.md#decisions) sets read and
      write the document. `sdfcore` holds the tables as branded-id tables.
- [x] **S2. Builder.** The builder's `.ts` files:
  1. An entry point that puts the API in scope, imports the model, and writes
     the `.sdfj` document
  2. Type definitions declaring the API as globals, matching the modeling API's
     declaration blocks
  3. The builder's checks from model evaluation
  4. A test per call pinning the entry the call writes
- [x] **S3. Build command.** `vxl sdf-doc build` with `--runtime` and profiles
      at `sdfDoc.build.profiles`. vxl embeds the builder and writes it to a
      temporary directory for each run. `vxl profile sdf-doc build list` lists
      the profiles.
- [x] **S4. Shapes.** A voxsmith operation reads the `.sdfj` document and
      evaluates every shape, each with a test over a small grid against its
      distance in model evaluation.
- [x] **S5. Steps.** The sampling, the boxes' rounding and clamp, the grid, the
      five steps, the document checks, and parts with their grids, hierarchy,
      places, offsets, and frames. A test pins a box with corners on cell
      corners filling exactly its cells.
- [x] **S6. Materials.** The named and custom properties with their checks,
      `shades`, and the patterns.
- [x] **S7. Libraries.** Libraries as [decision 17](README.md#decisions) sets
      them:
  1. The `names` table in sdfj, sdfcore, and the bridge
  2. The builder writing names, reading the libraries vxl hands it, and copying
     the entries a model uses
  3. `sdfDoc.build.libraries`, the build profiles' `libraries`, and `--library`
  4. vxl's built-in `materials` library with properties for every name the
     modeling API lists. A test checks every material with its default `shades`
- [x] **S8. Voxelize command.** `vxl sdf-doc voxelize` with the flags and
      profiles the modeling API lists, the voxj document, and the report under
      `--report`. `vxl profile sdf-doc voxelize list` lists the profiles.
- [x] **S9. Review profile.** A built-in `review` render profile imports the
      built-in `hero` view and adds orthographic `front`, `right`, and `top`
      views under the default `studio` rig. The render profile language's
      built-ins list `review`. The brightest library material keeps its shading
      short of white.
- [x] **S10. Integration commands.** Move every workspace binary's `completion`
      command to `integration completion print`. Update the tyt-meta templates
      that scaffold new crates and the READMEs that show the command.
- [x] **S11. Skill.** Write the skill's workflow as the README lays it out. Move
      the workflow and the modeling API into vxl. The modeling API links to
      other pages by their repository URLs.
      `vxl integration skill print vxl-model` prints the workflow and then the
      modeling API as one `SKILL.md`. Move model evaluation to
      `doc/ref/sdf-doc/` and the sdfj format into the sdfj crate's `docs/`.
- [x] **S12. Trials.** Run the prompts in `trials.md` through the skill, each in
      a fresh session. Log each prompt in `trials.md` with the passes it took,
      the failures, and the operations and report data Claude wanted and
      lacked. The log also notes where a model's colors read flat.
- [x] **S13. Round 1 fixes.** Answer round 1's [findings](trials.md#findings)
      before the next round:
  1. The fixes staged with the round, for findings 1, 8, 15, 17, 32, 40, and 54
  2. The owner's calls:
     1. `studio` lights from a white sky over a #404040 ground for findings 20,
        21, and 27
     2. `review` takes a bloom whose threshold reads an emission's brightest
        channel for finding 18. The modeling API gives the strengths that keep a
        glow's hue. The library's light group follows them
     3. `grain` rings its axis for finding 32
     4. `box`, `boxFrame`, and `rect` take their corners in either order for
        finding 45
     5. Every binary takes `--version` for finding 57
  3. The skill text for findings 2, 3, 4, 6, 7, 9, 10, 13, 14, 16, 19, 28, 30,
     34, 36, 44, and 55
  4. A look at findings 5, 12, 31, 37, 50, 52, and 56:
     1. The neutral fill answers finding 5 and the rope in finding 31. The
        library's `grass` turns less saturated for the rest of finding 31
     2. Skill lines answer findings 12, 37, 52, and 56
     3. Finding 50 joins phase 2's candidates
- [x] **S14. Rounds 2 to 6.** Run the prompts again with S13's changes and log
      the round beside round 1. A finding S13 answered reopens when round 2
      still shows it. [Round 2](trials-round-2.md) reopens 27 of the 36 findings
      S13 answered. [Round 3](trials-round-3.md) runs the prompts after a
      shorter skill and fixes to vxl and the harness. It clears 9 of the 13
      findings those changes answered. [Round 4](trials-round-4.md) clears 2 of
      the 5 findings its changes answered. [Round 5](trials-round-5.md) clears 3
      of the 7 findings its changes answered and leaves a fourth untested.
      [Round 6](trials-round-6.md) clears 1 of the 3 findings its changes
      answered and leaves a second untested.
- [x] **S15. Phase 2 design.** Write the phase 2 steps from the six rounds and
      revise the README.

## Phase 2: follow-ups

The trial rounds set phase 2's steps. The steps cite findings by the numbers
[round 7](trials-round-7.md#findings) gives them. A step that adds a call also
adds the call's entry to the sdfj format. The candidates the trials asked for
rarely or never [wait for a reason](README.md#waiting-for-a-reason).

- [x] **S16. Report.** The report data the rounds asked for, each printed only
      where it applies:
  1. A step line counts the face-connected pieces its live cells form when they
     form more than one for findings 10 and 24. A cell a later step recolored
     still counts. A piece line lists every step whose live cells reach the
     piece for finding 33
  2. A step line that lost cells lists the later steps that took them for
     findings 7, 55, and 122
  3. A `paint` or `coat` line lists the steps whose cells it recolored for
     finding 34
  4. `exposed` counts an opaque cell beside a material that lets light through
     for finding 51
  5. A part line gives the part's pivot for finding 42. A part's steps print
     under its first place only for finding 23
  6. The API doc points to `vxl palette show` for a custom property's values
     for finding 39. `vxl node list --show-transforms` already prints the nodes
- [x] **S17. Review sheet.** The contact sheet moves out of the
      [rendering follow-ups](../voxel-rendering-followups/README.md#contact-sheet)
      for findings 8, 17, and 19:
  1. `review` tiles its four views and a view from behind into one PNG
  2. A part's `rotation` poses its node for finding 35. The render's `--rotate`
     rotates a part for one render
- [x] **S18. Library colors.** Retune the `materials` library and `shades` under
      the review rig:
  1. Dark woods, stone, and bark keep their pattern visible for finding 5
  2. Oak, walnut, rope, and grass read the hue their names give for findings 31
     and 69. The metals wait for the rendering follow-ups'
     [reflections](../voxel-rendering-followups/README.md#reflections) for
     findings 5 and 20
  3. Every shade stays in the sRGB gamut and near the base's hue for findings
     15 and 68
  4. The default `spread` stops short of the swings that read as camouflage for
     finding 65
- [x] **S19. Round 7.** Run the prompts after S16 to S18 and log the round
      beside round 6. The round compares its passes and cost with round 5's
      because round 6 ran the dropped review. [Round 7](trials-round-7.md)
      matches round 5's passes and cost.
- [x] **S20. Spills.** Options that keep paint on its target:
  1. `onto` limits a `paint` or a `coat` to the points that listed `add` steps
     still own for finding 34. A point belongs to the last `add` whose shape
     holds it unless a later `carve` emptied it. `onto` takes the steps as
     values, and each has to come earlier in the same list
  2. `coat` reads the part's shapes so far in place of the grid. A point takes
     the coat when the point `depth` meters toward a listed side lies outside
     those shapes. `depth` has no default, and vxl errors on one under one voxel
- [x] **S21. Curves.** Curves the rounds built by hand:
  1. `spline` draws a smooth 2D curve through points as an outline or a stroke
     for finding 87
  2. `sweep` runs a 2D profile along a 3D path of points and can taper it.
     `helix` gives a coiled path for stairs, springs, and spirals. Both answer
     finding 25
- [x] **S22. Meters only.** sdfj stops counting voxels, for decision 20:
  1. Each pattern takes its defining length after its materials: a period for
     `bands` and `grain`, a scale for `noise`, and a size for `checker`,
     `cells`, and `speckle`. `speckle` picks per world-aligned cube of its size
  2. `grain`'s `warp` defaults to 0.75 of its period
  3. The `cells` border takes a width
  4. `set` takes a size and fills a cube of that size centered on each point
  5. Decision 10 drops its cell counts, and decision 3 recasts `set`
  6. tyt-assets' models move to the new arguments
  7. The owner can move models in other repositories, such as the voxel art
     repository, to the new arguments
- [x] **S23. Content interning.** sdfj-builder writes one table entry per
      distinct content, so a value a model builds twice reads once:
  1. Entries key on their written fields, children first
  2. A vxl flag turns it off and keeps one entry per JS object
  3. A part stays its own object, whatever its steps hold
  4. The build time of a large scene gets measured, and multithreading follows
     if interning slows it
- [x] **S24. Build variables.** A model reads build variables through `vars`,
      which `vxl sdf-doc build` fills from `--var <name>=<value>` and a build
      profile's `vars`. Reading a variable the build lacks errors. The `.sdfj`
      document holds only the values the variables gave. No variable stands
      for the voxel size
- [x] **S25. Variable imports.** A build profile's `varsFrom` and
      `--vars-from` import other profiles' build variables. The variables land
      from the profile's imports, its `vars`, each `--vars-from`, and each
      `--var` in turn, a later variable replacing an earlier one of its name
- [x] **S26. Rotation.** Rotated shapes for finding 22:
  1. Under `--flatten`, vxl samples a rotated part at its angle on the root
     part's grid. The sampling takes the part's rotation and offset as inputs
  2. A negative `scale` factor reflects a shape across its axis without
     keeping the original
  3. A part marked `joint` keeps its node under `--flatten` and voxelizes
     unrotated, leaving an animation a node to turn
- [x] **S27. Shared nodes.** A node the `.sdfj` document shares stays one voxj
      node with several parents, along with its subtree and objects:
  1. Under `--frame world`, places whose shifts differ by whole voxels share
     the node. Places that would sample differently error. The error suggests
     whole-voxel offsets, `--frame local`, or a fresh part per place
  2. The `voxels` child of an off-lattice pivot is shared with its node
  3. `--flatten` still expands every place
  4. The report keeps a line per place
  5. Decision 18 drops "Each place writes a node"
- [x] **S28. Neutral sdfj spec.** `sdf-json-file-format.md` sets the format
      for any tool, in the style of the voxel json spec:
  1. The spec opens with its design objectives and coordinate system and
     mentions neither vxl nor the builder
  2. Each table takes annotated JSONC that gives every key's meaning. The
     spec sets what each entry computes at a point, and the distances,
     booleans, transforms, modifiers, noise, patterns, steps, and material
     decoding move there from model evaluation
  3. A validation section states the rules every reader applies, the argument
     checks included
  4. The examples hold plain JSONC
  5. The builder's writing rules move to the builder's README. Model
     evaluation keeps vxl's settings, libraries, sampling, arithmetic, bounds,
     grid, voxj document, report, and checks
- [x] **S29. Builder API.** The TypeScript API moves into sdfj-builder:
  1. sdfj-builder's `docs/typescript-api.md` lists every call with its
     signature and effect. The vxl-model skill embeds it between the workflow
     and the modeling API
  2. The sdfj-builder README states how the builder takes a TypeScript model
     and writes an `.sdfj` document, with the builder's writing rules and checks
  3. The modeling API keeps how vxl runs a model and how the model lands on the
     grid, and the other docs point to the TypeScript API for the calls
  4. A doc review asks questions the old docs answer, trick questions included,
     and scores rewrites of the new docs on them
  5. A trial round of 30 shows each model beside its image from the last round
- [x] **S30. Slash warnings.** A voxcore check lists the nodes and objects
      whose names hold `/`, which makes their joined paths ambiguous. vxl runs
      the check after every conversion into voxcore, from a file read or from a
      voxelize, and warns with a pointer to `--select-index`. Names pass
      through unchanged
- [x] **S31. New calls.** More calls the rounds built by hand:
  1. `text` lays out a string as a 2D shape for finding 47. The font comes
     from a file or an installed font that a flag or `.vxlconfig` provides,
     never from vxl. The build stores each glyph used as an exact outline of
     lines and quadratic curves. Cubic curves split into quadratics at import
  2. `bricks` lays courses of bricks with mortar joints for findings 37 and 92
  3. `random` gives the model a seeded generator for placements and sizes for
     findings 26 and 91
- [x] **S32. Round 7 fixes.** Answer round 7's
      [findings](trials-round-7.md#findings) before the next round:
  - [x] The review sheet keeps `<stem>.png` whatever views the command adds
        for finding 126
  - [x] The report ends with a block that repeats its `0 kept`, `0 exposed`,
        extra piece, and broken-step lines for finding 4. The skill's done line
        points to the block
  - [x] A rotated part's line and the model line give posed world bounds and
        the world pivot for finding 128
  - [x] `coat` takes a slope limit for finding 135, and `gradient` takes its
        range in either order for finding 136
- [x] **S33. Measured colors.** The library's colors take measured values
      for finding 13. The decisions log cites each color's source. The metals
      take their measured reflectances ahead of S34. Where two real materials
      share a color, grain or `shades` tells them apart
- [x] **S34. Reflections.** A metal reflects the scene along its mirror
      direction for findings 5 and 20, as the rendering follow-ups'
      [reflections](../voxel-rendering-followups/README.md#reflections)
      section lays out. `studio` takes a light ground for a metal's sides to
      reflect. The review renders confirm S33's metals
- [ ] **S35. Round 8.** Run the prompts after S20 to S34 and log the round
      beside round 7. The judging records whether sessions skimmed past the
      `check` lines of steps meant to be in pieces. The answer decides whether
      the block keeps those steps.
